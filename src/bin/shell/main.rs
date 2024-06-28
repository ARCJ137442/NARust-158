use anyhow::Result;
use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
use narust_158::{control::DEFAULT_PARAMETERS, inference::InferenceEngine, vm::Launcher};
use navm::{
    cmd::Cmd,
    output::Output,
    vm::{VmLauncher, VmRuntime},
};

pub fn launcher_void() -> impl VmLauncher {
    Launcher::new("nar_158", DEFAULT_PARAMETERS, InferenceEngine::VOID)
}

pub fn launcher_echo() -> impl VmLauncher {
    Launcher::new("nar_158", DEFAULT_PARAMETERS, InferenceEngine::ECHO)
}

fn create_runtime() -> Result<impl VmRuntime> {
    let vm = launcher_echo();
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
        match Cmd::parse(input) {
            Ok(cmd) => runtime.input_cmd(cmd)?,
            Err(err) => eprintln!("NAVM cmd parse error: {err}"),
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
        output => println!("{}", output.to_json_string()),
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

pub fn main() -> Result<()> {
    let runtime = create_runtime()?;
    shell(runtime, shell_iter_stdin())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_term_stability() -> Result<()> {
        const INPUTS: &str = r#"
            nse <{tim} --> (/,livingIn,_,{graz})>. %0%
            cyc 100
            nse <<(*,$1,sunglasses) --> own> ==> <$1 --> [aggressive]>>.
            nse <(*,{tom},sunglasses) --> own>.
            nse <<$1 --> [aggressive]> ==> <$1 --> murder>>.
            nse <<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>.
            nse <{?who} --> murder>?
            nse <{tim} --> (/,livingIn,_,{graz})>.
            nse <{tim} --> (/,livingIn,_,{graz})>. %0%
            cyc 100
            nse <<(*,$1,sunglasses) --> own> ==> <$1 --> [aggressive]>>.
            nse <(*,{tom},(&,[black],glasses)) --> own>.
            nse <<$1 --> [aggressive]> ==> <$1 --> murder>>.
            nse <<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>.
            nse <sunglasses --> (&,[black],glasses)>.
            nse <{?who} --> murder>?
            nse <(*,toothbrush,plastic) --> made_of>.
            nse <(&/,<(*,$1,plastic) --> made_of>,(^lighter,{SELF},$1)) =/> <$1 --> [heated]>>.
            nse <<$1 --> [heated]> =/> <$1 --> [melted]>>.
            nse <<$1 --> [melted]> <|> <$1 --> [pliable]>>.
            nse <(&/,<$1 --> [pliable]>,(^reshape,{SELF},$1)) =/> <$1 --> [hardened]>>.
            nse <<$1 --> [hardened]> =|> <$1 --> [unscrewing]>>.
            nse <toothbrush --> object>.
            nse (&&,<#1 --> object>,<#1 --> [unscrewing]>)!
            nse <{SELF} --> [hurt]>! %0%
            nse <{SELF} --> [hurt]>. :|: %0%
            nse <(&/,<(*,{SELF},wolf) --> close_to>,+1000) =/> <{SELF} --> [hurt]>>.
            nse <(*,{SELF},wolf) --> close_to>. :|:
            nse <(&|,(^want,{SELF},$1,FALSE),(^anticipate,{SELF},$1)) =|> <(*,{SELF},$1) --> afraid_of>>.
            nse <(*,{SELF},?what) --> afraid_of>?
            nse <a --> A>. :|: %1.00;0.90%
            cyc 8
            nse <b --> B>. :|: %1.00;0.90%
            cyc 8
            nse <c --> C>. :|: %1.00;0.90%
            cyc 8
            nse <a --> A>. :|: %1.00;0.90%
            cyc 100
            nse <b --> B>. :|: %1.00;0.90%
            cyc 100
            nse <?1 =/> <c --> C>>?
            nse <(*,cup,plastic) --> made_of>.
            nse <cup --> object>.
            nse <cup --> [bendable]>.
            nse <toothbrush --> [bendable]>.
            nse <toothbrush --> object>.
            nse <(&/,<(*,$1,plastic) --> made_of>,(^lighter,{SELF},$1)) =/> <$1 --> [heated]>>.
            nse <<$1 --> [heated]> =/> <$1 --> [melted]>>.
            nse <<$1 --> [melted]> <|> <$1 --> [pliable]>>.
            nse <(&/,<$1 --> [pliable]>,(^reshape,{SELF},$1)) =/> <$1 --> [hardened]>>.
            nse <<$1 --> [hardened]> =|> <$1 --> [unscrewing]>>.
            nse (&&,<#1 --> object>,<#1 --> [unscrewing]>)!
            cyc 20000"#;
        let runtime = create_runtime()?;
        // let inputs = shell_iter_stdin();
        let inputs = shell_iter_inputs(
            INPUTS
                .lines()
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string()),
        );
        shell(runtime, inputs)?;
        Ok(())
    }
}
