//! 标准NAVM「批处理」程序
//! * 🚩以[NAVM指令](Cmd)作为输入，以JSON格式输出[NAVM输出](Output)
//! * 🎯对接BabelNAR「原生转译器」接口

use anyhow::Result;
use narust_158::{
    control::DEFAULT_PARAMETERS,
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    vm::Launcher,
};
use navm::{
    cmd::Cmd,
    output::Output,
    vm::{VmLauncher, VmRuntime},
};

fn create_runtime() -> Result<impl VmRuntime> {
    // * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        match_task_and_belief,
        reason,
    );
    let vm = Launcher::new("demo_158", DEFAULT_PARAMETERS, ENGINE);
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
        match Cmd::parse(input) {
            Ok(cmd) => runtime.input_cmd(cmd)?,
            Err(err) => eprintln!("NAVM cmd parse error: {err}"),
        }
        // out
        while let Some(output) = runtime.try_fetch_output()? {
            batch_output(output);
        }
    }
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
