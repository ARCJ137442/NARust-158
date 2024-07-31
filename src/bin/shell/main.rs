use anyhow::Result;
use nar_dev_utils::ResultBoost;
use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
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
use std::io::{stdout, Write};

pub fn launcher_void() -> impl VmLauncher {
    Launcher::new("nar_158", DEFAULT_PARAMETERS, InferenceEngine::VOID)
}

pub fn launcher_echo() -> impl VmLauncher {
    Launcher::new("nar_158", DEFAULT_PARAMETERS, InferenceEngine::ECHO)
}

pub fn launcher_dev() -> impl VmLauncher {
    // * 🚩【2024-07-09 16:52:40】目前除了「概念推理」均俱全
    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        match_task_and_belief,
        reason,
    );
    Launcher::new("nar_158", DEFAULT_PARAMETERS, ENGINE)
}

fn create_runtime() -> Result<impl VmRuntime> {
    let vm = launcher_dev();
    vm.launch()
}

fn shell(
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
            // 正常获取但continue
            Some(Ok(None)) => continue,
            // 正常获取
            Some(Ok(Some(input))) => input,
        };
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let cmd = 'cmd: {
            // 纯数字⇒尝试默认成`CYC`指令
            if let Ok(n) = input.parse::<usize>() {
                break 'cmd Some(Cmd::CYC(n));
            }
            // 若能解析成词法Narsese任务⇒尝试默认成`NSE`指令
            if let Ok(Ok(task)) = FORMAT_ASCII
                .parse(input)
                .map(|value| value.try_into_task_compatible())
            {
                break 'cmd Some(Cmd::NSE(task));
            }
            // 最后再考虑作为NAVM指令解析
            Cmd::parse(input).ok_or_run(|err| eprintln!("NAVM cmd parse error: {err}"))
        };
        if let Some(cmd) = cmd {
            runtime.input_cmd(cmd)?;
        }
        // out
        while let Some(output) = runtime.try_fetch_output()? {
            shell_output(output);
        }
    }
}

fn shell_output(output: Output) {
    use Output::*;
    match &output {
        // 带Narsese输出
        IN { content, narsese }
        | OUT {
            content_raw: content,
            narsese,
        }
        | ANSWER {
            content_raw: content,
            narsese,
        }
        | ACHIEVED {
            content_raw: content,
            narsese,
        } => match narsese {
            Some(narsese) => {
                println!("[{}] {}", output.get_type(), FORMAT_ASCII.format(narsese))
            }
            None => println!("[{}] {}", output.get_type(), content),
        },
        // 仅消息
        ERROR {
            description: content,
        }
        | INFO { message: content }
        | COMMENT { content }
        | TERMINATED {
            description: content,
        }
        | OTHER { content } => println!("[{}] {}", output.get_type(), content),
        // 其它
        output => {
            println!("{}", output.to_json_string());
            stdout().flush().unwrap();
        }
    }
}

/// 固定的一系列输入
pub fn shell_iter_inputs(
    inputs: impl IntoIterator<Item = String>,
) -> impl Iterator<Item = Result<Option<String>>> {
    inputs.into_iter().map(|content| Ok(Some(content)))
}

/// 从「标准输入」读取输入
pub fn shell_iter_stdin() -> impl Iterator<Item = Result<Option<String>>> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    std::iter::from_fn(move || {
        // 获取一个
        let bytes = match stdin.read_line(&mut buffer) {
            Ok(b) => b,
            Err(e) => return Some(Err(e.into())),
        };
        if bytes == 0 {
            return Some(Ok(None));
        }
        // clear
        let input = buffer.clone();
        buffer.clear();
        Some(Ok(Some(input)))
    })
}

/// 设置虚拟机到「最大音量」
/// * 🎯使虚拟机得以输出尽可能详尽的信息
pub fn set_max_volume(vm: &mut impl VmRuntime) -> Result<()> {
    vm.input_cmd(Cmd::VOL(100))?;
    vm.try_fetch_output()?; // 📌丢掉其输出
    Ok(())
}

pub fn main() -> Result<()> {
    // * 🚩创建
    let runtime = create_runtime()?;
    // * 🚩音量
    // * 🚩【2024-07-31 23:20:33】现不再默认最大音量
    // set_max_volume(&mut runtime)?;
    // * 🚩交互
    shell(runtime, shell_iter_stdin())?;
    Ok(())
}
