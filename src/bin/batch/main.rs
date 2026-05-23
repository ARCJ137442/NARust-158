//! 标准NAVM「批处理」程序
//! * 🚩以[NAVM指令](Cmd)作为输入，以JSON格式输出[NAVM输出](Output)
//! * 🎯对接BabelNAR「原生转译器」接口

use anyhow::Result;
use narust_158::{
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    parameters::DEFAULT_PARAMETERS,
    vm::alpha::{LauncherAlpha, SavCallback},
};
use navm::{cmd::Cmd, output::Output, vm::VmLauncher, vm::VmRuntime};
use std::{io::Write, path::Path};

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
            // 提前解释输出
            if let Some(output) = batch_intercept_output(output)? {
                batch_output(output);
            }
        }
    }
}

/// 从输入中「提前解释」指令
/// * 💡可以从中对指令作预处理
///   * 📄绕过生硬的NAVM指令语法，像OpenNARS那样直接输入Narsese与推理步数
///   * 📄截获解析出的`SAV` `LOA`等指令，解释为其它指令语法
///     * 💡如：`LOA`指令⇒前端请求文件并读取内容⇒内联到新的`LOA`中⇒虚拟机Alpha实现内容加载
fn interpret_cmd(input: &str) -> Option<Cmd> {
    // 尝试作为普通NAVM指令解析
    if let Ok(cmd) = Cmd::parse(input) {
        match cmd {
            // `LOA`指令转译：路径→文件内容
            Cmd::LOA { target, path } => {
                let data = match try_load_file_content(path) {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("NAVM LOA cmd load error: {err}");
                        return None;
                    }
                };
                return Some(Cmd::LOA { target, path: data });
            }
            // 自定义指令：忽略
            // * 避免解析范围的扩大，导致输入`A.`不通过
            Cmd::Custom { .. } => {}
            // 其它⇒解析成功
            _ => return Some(cmd),
        }
    }
    // 最终仍然解析失败
    eprintln!("NAVM cmd parse error: {input:?}");
    None
}

/// 尝试读取本地文件，将内容作为`LOA`指令的path参数
fn try_load_file_content(path: impl AsRef<str>) -> anyhow::Result<String> {
    // * 🚩尝试读取本地文件
    let path = path.as_ref();
    if Path::new(path).exists() {
        let content = std::fs::read_to_string(path)?;
        return Ok(content);
    }
    Err(anyhow::anyhow!("File not found: {path}"))
}

/// 终端拦截输出
/// * 🎯根据「有路径的SAV」输出文件
fn batch_intercept_output(output: Output) -> anyhow::Result<Option<Output>> {
    // * 🚩拦截「SAV」回调
    let output = match output.try_into_sav_callback() {
        // 空路径⇒不保存⇒重组回「消息」并继续（输出到终端）
        Ok((path, data)) if path.is_empty() => Output::format_sav_callback(path, data),
        // 有路径⇒保存到文件
        Ok((path, data)) => {
            // * 🚩尝试保存文件
            let result = save_file(&path, &data);
            // * 🚩将终端输出重定向到文件
            let message = match result {
                // * 🚩生成「已保存」的消息
                Ok(..) => format!("Data has been saved to {path:?} with {} bytes", data.len()),
                // * 🚩或报错消息
                Err(e) => format!("Failed to save data to {path:?}! Error: {e}"),
            };
            // * 🚩替换为「已保存」的回显
            let out = Output::INFO { message };
            return Ok(Some(out));
        }
        // 未消耗⇒继续
        Err(output) => output,
    };
    // 正常未消耗输出
    Ok(Some(output))
}

/// 路径+数据→保存文件
fn save_file(path: impl Into<String>, data: &str) -> Result<()> {
    use std::fs::File;
    let mut file = File::create(path.into())?;
    file.write_all(data.as_bytes())?;
    Ok(())
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
