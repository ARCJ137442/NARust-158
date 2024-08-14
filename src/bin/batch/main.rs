//! 标准NAVM「批处理」程序
//! * 🚩以[NAVM指令](Cmd)作为输入，以JSON格式输出[NAVM输出](Output)
//! * 🎯对接BabelNAR「原生转译器」接口

use anyhow::Result;
use nar_dev_utils::ResultBoost;
use narust_158::{
    control::DEFAULT_PARAMETERS,
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    vm::alpha::LauncherAlpha,
};
use navm::{cmd::Cmd, output::Output, vm::VmLauncher, vm::VmRuntime};

fn create_runtime() -> Result<impl VmRuntime> {
    // * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        match_task_and_belief,
        reason,
    );
    let vm = LauncherAlpha::new("demo_158", DEFAULT_PARAMETERS, ENGINE);
    vm.launch()
}

fn batch(
    mut runtime: impl VmRuntime,
    mut inputs: impl Iterator<Item = Result<Option<String>>>,
) -> Result<()> {
    loop {
        // in
        let input = match inputs.next() {
            // 正常结束
            None => return Ok(()),
            // 异常结束
            Some(Err(e)) => return Err(e),
            // EOF
            Some(Ok(None)) => {
                eprintln!("Program exited with EOF.");
                break Ok(());
            }
            // 正常获取
            Some(Ok(Some(input))) => input,
        };
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        // 尝试预先解释输入
        if let Some(cmd) = interpret_cmd(input) {
            runtime.input_cmd(cmd)?;
        }
        // out
        while let Some(output) = runtime.try_fetch_output()? {
            batch_output(output);
        }
    }
}

/// 从输入中「提前解释」指令
/// * 💡可以从中对指令作预处理
///   * 📄绕过生硬的NAVM指令语法，像OpenNARS那样直接输入Narsese与推理步数
///   * 📄截获解析出的`SAV` `LOA`等指令，解释为其它指令语法
///     * 💡如：`LOA`指令⇒前端请求文件并读取内容⇒内联到新的`LOA`中⇒虚拟机Alpha实现内容加载
fn interpret_cmd(input: &str) -> Option<Cmd> {
    // 目前只作为NAVM指令解析
    Cmd::parse(input).ok_or_run(|err| eprintln!("NAVM cmd parse error: {err}"))
}

/// 输出：仅打印JSON
fn batch_output(output: Output) {
    println!("{}", output.to_json_string());
}

/// 固定的一系列输入
pub fn batch_iter_inputs(
    inputs: impl IntoIterator<Item = String>,
) -> impl Iterator<Item = Result<Option<String>>> {
    inputs.into_iter().map(|content| Ok(Some(content)))
}

/// 从「标准输入」读取输入
pub fn batch_iter_stdin() -> impl Iterator<Item = Result<Option<String>>> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    std::iter::from_fn(move || {
        // 获取一个
        let bytes = match stdin.read_line(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Some(Err(e.into())),
        };
        if bytes == 0 {
            // * 🚩【2024-07-31 23:33:20】此处实乃EOF也
            // * 🔗参考「Rust如何检测EOF」：https://stackoverflow.com/questions/27475113/how-to-check-for-eof-with-read-line
            return Some(Ok(None));
        }
        // clear
        let input = buffer.clone();
        buffer.clear();
        Some(Ok(Some(input)))
    })
}

pub fn main() -> Result<()> {
    // * 🚩创建
    let runtime = create_runtime()?;
    // * 🚩交互
    batch(runtime, batch_iter_stdin())?;
    Ok(())
}
