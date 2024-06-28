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

    /// 测试多行NAVM指令（文本形式）输入
    /// * 🚩仅测试文本输入（稳定性），不负责捕获输出等额外操作
    fn test_line_inputs(inputs: &str) -> Result<()> {
        let runtime = create_runtime()?;
        let inputs = shell_iter_inputs(
            inputs
                .lines()
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string()),
        );
        shell(runtime, inputs)?;
        Ok(())
    }

    #[test]
    fn long_term_stability() -> Result<()> {
        test_line_inputs(
            r#"
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
            cyc 20000"#,
        )
    }

    #[test]
    fn logical_stability() -> Result<()> {
        test_line_inputs(
            r#"
            rem 1-6 stability

            rem file: '1.0.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %0.10;0.60%
            cyc 100

            rem file: '1.1.nal'
            nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            cyc 100

            rem file: '1.2.nal'
            nse $0.80;0.80;0.95$ <sport --> competition>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <chess --> competition>. %0.90;0.90%
            cyc 100

            rem file: '1.3.nal'
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
            cyc 100

            rem file: '1.4.nal'
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%
            cyc 100

            rem file: '1.5.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <swimmer --> bird>?
            cyc 100

            rem file: '1.6.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <bird --> swimmer>?
            cyc 100

            rem file: '1.7.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.80%
            nse $0.90;0.80;1.00$ <?x --> swimmer>?
            cyc 100

            rem file: '1.8.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.80%
            nse $0.90;0.80;1.00$ <?1 --> swimmer>?
            cyc 100

            rem file: '2.0.nal'
            nse $0.80;0.80;0.95$ <robin <-> swan>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin <-> swan>. %0.10;0.60%
            cyc 100

            rem file: '2.1.nal'
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
            cyc 100

            rem file: '2.10.nal'
            nse $0.80;0.80;0.95$ <Birdie <-> Tweety>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <{Birdie} <-> {Tweety}>?
            cyc 100

            rem file: '2.11.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <bird <-> swan>?
            cyc 100

            rem file: '2.12.nal'
            nse $0.80;0.80;0.95$ <bird <-> swan>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <swan --> bird>?
            cyc 100

            rem file: '2.13.nal'
            nse $0.80;0.80;0.95$ <Tweety {-- bird>. %1.00;0.90%
            cyc 100

            rem file: '2.14.nal'
            nse $0.80;0.80;0.95$ <raven --] black>. %1.00;0.90%
            cyc 100

            rem file: '2.15.nal'
            nse $0.80;0.80;0.95$ <Tweety {-] yellow>. %1.00;0.90%
            cyc 100

            rem file: '2.16.nal'
            nse $0.80;0.80;0.95$ <{Tweety} --> {Birdie}>. %1.00;0.90%
            cyc 100

            rem file: '2.17.nal'
            nse $0.80;0.80;0.95$ <[smart] --> [bright]>. %1.00;0.90%
            cyc 100

            rem file: '2.18.nal'
            nse $0.80;0.80;0.95$ <{Birdie} <-> {Tweety}>. %1.00;0.90%
            cyc 100

            rem file: '2.19.nal'
            nse $0.80;0.80;0.95$ <[bright] <-> [smart]>. %1.00;0.90%
            cyc 100

            rem file: '2.2.nal'
            nse $0.80;0.80;0.95$ <bird --> swimmer>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <{?1} --> swimmer>?
            cyc 100

            rem file: '2.3.nal'
            nse $0.80;0.80;0.95$ <sport --> competition>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <chess --> competition>. %0.90;0.90%
            cyc 100

            rem file: '2.4.nal'
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%
            cyc 100

            rem file: '2.5.nal'
            nse $0.80;0.80;0.95$ <gull --> swimmer>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%
            cyc 100

            rem file: '2.6.nal'
            nse $0.80;0.80;0.95$ <robin <-> swan>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <gull <-> swan>. %1.00;0.90%
            cyc 100

            rem file: '2.7.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <bird --> swan>. %0.10;0.90%
            cyc 100

            rem file: '2.8.nal'
            nse $0.80;0.80;0.95$ <bright <-> smart>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <[smart] --> [bright]>?
            cyc 100

            rem file: '2.9.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <bird <-> swan>. %0.10;0.90%
            cyc 100

            rem file: '3.0.nal'
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.80;0.90%
            cyc 100

            rem file: '3.1.nal'
            nse $0.80;0.80;0.95$ <sport --> competition>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <chess --> competition>. %0.80;0.90%
            cyc 100

            rem file: '3.10.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <swan --> (-,swimmer,bird)>?
            cyc 100

            rem file: '3.11.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <(~,swimmer,swan) --> bird>?
            cyc 100

            rem file: '3.12.nal'
            nse $0.80;0.80;0.95$ <robin --> (&,bird,swimmer)>. %0.90;0.90%
            cyc 100

            rem file: '3.13.nal'
            nse $0.80;0.80;0.95$ <robin --> (-,bird,swimmer)>. %0.90;0.90%
            cyc 100

            rem file: '3.14.nal'
            nse $0.80;0.80;0.95$ <(|,boy,girl) --> youth>. %0.90;0.90%
            cyc 100

            rem file: '3.15.nal'
            nse $0.80;0.80;0.95$ <(~,boy,girl) --> [strong]>. %0.90;0.90%
            cyc 100

            rem file: '3.2.nal'
            nse $0.80;0.80;0.95$ <robin --> (|,bird,swimmer)>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%
            cyc 100

            rem file: '3.3.nal'
            nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> (-,mammal,swimmer)>. %0.00;0.90%
            cyc 100

            rem file: '3.4.nal'
            nse $0.80;0.80;0.95$ <planetX --> {Mars,Pluto,Venus}>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <planetX --> {Pluto,Saturn}>. %0.70;0.90%
            cyc 100

            rem file: '3.5.nal'
            nse $0.80;0.80;0.95$ <planetX --> {Mars,Pluto,Venus}>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <planetX --> {Pluto,Saturn}>. %0.10;0.90%
            cyc 100

            rem file: '3.6.nal'
            nse $0.80;0.80;0.95$ <bird --> animal>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <(&,bird,swimmer) --> (&,animal,swimmer)>?
            cyc 100

            rem file: '3.7.nal'
            nse $0.80;0.80;0.95$ <bird --> animal>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <(-,swimmer,animal) --> (-,swimmer,bird)>?
            cyc 100

            rem file: '3.8.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <swan --> (|,bird,swimmer)>?
            cyc 100

            rem file: '3.9.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            nse $0.90;0.80;1.00$ <(&,swan,swimmer) --> bird>?
            cyc 100

            rem file: '4.0.nal'
            nse $0.80;0.80;0.95$ <(*,acid,base) --> reaction>. %1.00;0.90%
            cyc 100

            rem file: '4.1.nal'
            nse $0.80;0.80;0.95$ <acid --> (/,reaction,_,base)>. %1.00;0.90%
            cyc 100

            rem file: '4.2.nal'
            nse $0.80;0.80;0.95$ <base --> (/,reaction,acid,_)>. %1.00;0.90%
            cyc 100

            rem file: '4.3.nal'
            nse $0.80;0.80;0.95$ <neutralization --> (*,acid,base)>. %1.00;0.90%
            cyc 100

            rem file: '4.4.nal'
            nse $0.80;0.80;0.95$ <(\\,neutralization,_,base) --> acid>. %1.00;0.90%
            cyc 100

            rem file: '4.5.nal'
            nse $0.80;0.80;0.95$ <(\\,neutralization,acid,_) --> base>. %1.00;0.90%
            cyc 100

            rem file: '4.6.nal'
            nse $0.80;0.80;0.95$ <bird --> animal>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <(*,bird,plant) --> ?x>?
            cyc 100

            rem file: '4.7.nal'
            nse $0.80;0.80;0.95$ <neutralization --> reaction>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <(\\,neutralization,acid,_) --> ?x>?
            cyc 100

            rem file: '4.8.nal'
            nse $0.80;0.80;0.95$ <soda --> base>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <(/,neutralization,_,base) --> ?x>?
            cyc 100

            rem file: '5.0.nal'
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %0.00;0.60%
            cyc 100

            rem file: '5.1.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '5.10.nal'
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.80;0.90%
            cyc 100

            rem file: '5.11.nal'
            nse $0.80;0.80;0.95$ <<robin --> animal> <=> <robin --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.90;0.90%
            cyc 100

            rem file: '5.12.nal'
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %0.90;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.90;0.90%
            cyc 100

            rem file: '5.13.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.90;0.90%
            cyc 100

            rem file: '5.14.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %0.90;0.90%
            cyc 100

            rem file: '5.15.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> (&&,<robin --> animal>,<robin --> [flying]>)>. %0.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %1.00;0.90%
            cyc 100

            rem file: '5.16.nal'
            nse $0.80;0.80;0.95$ (&&,<robin --> [flying]>,<robin --> swimmer>). %0.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%
            cyc 100

            rem file: '5.17.nal'
            nse $0.80;0.80;0.95$ (||,<robin --> [flying]>,<robin --> swimmer>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> swimmer>. %0.00;0.90%
            cyc 100

            rem file: '5.18.nal'
            nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%
            nse $0.90;0.80;1.00$ (||,<robin --> [flying]>,<robin --> swimmer>)?
            cyc 100

            rem file: '5.19.nal'
            nse $0.90;0.90;0.86$ (&&,<robin --> swimmer>,<robin --> [flying]>). %0.90;0.90%
            cyc 100

            rem file: '5.2.nal'
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            cyc 100

            rem file: '5.20.nal'
            nse $0.80;0.80;0.95$ (--,<robin --> [flying]>). %0.10;0.90%
            cyc 100

            rem file: '5.21.nal'
            nse $0.80;0.80;0.95$ <robin --> [flying]>. %0.90;0.90%
            nse $0.90;0.80;1.00$ (--,<robin --> [flying]>)?
            cyc 100

            rem file: '5.22.nal'
            nse $0.80;0.80;0.95$ <(--,<robin --> bird>) ==> <robin --> [flying]>>. %0.10;0.90%
            nse $0.90;0.80;1.00$ <(--,<robin --> [flying]>) ==> <robin --> bird>>?
            cyc 100

            rem file: '5.23.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>. %1.00;0.90%

            nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%
            cyc 100

            rem file: '5.24.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> [chirping]>,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> bird>>. %1.00;0.90%

            nse $0.80;0.80;0.95$ <robin --> [flying]>. %1.00;0.90%
            cyc 100

            rem file: '5.25.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> bird>,<robin --> [living]>) ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '5.26.nal'
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(&&,<robin --> swimmer>,<robin --> [flying]>) ==> <robin --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '5.27.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> [with_wings]>,<robin --> [chirping]>) ==> <robin --> bird>>. %1.00;0.90%

            nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>,<robin --> [chirping]>) ==> <robin --> bird>>. %1.00;0.90%

            cyc 100

            rem file: '5.28.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> [with_wings]>) ==> <robin --> [living]>>. %0.90;0.90%

            nse $0.80;0.80;0.95$ <(&&,<robin --> [flying]>,<robin --> bird>) ==> <robin --> [living]>>. %1.00;0.90%
            cyc 100

            rem file: '5.29.nal'
            nse $0.80;0.80;0.95$ <(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> [with_beak]>>. %0.90;0.90%
            cyc 100

            rem file: '5.3.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.80;0.90%
            cyc 100

            rem file: '5.4.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %0.80;0.90%
            cyc 100

            rem file: '5.5.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            cyc 100

            rem file: '5.6.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %0.70;0.90%
            nse $0.80;0.80;0.95$ <robin --> animal>. %1.00;0.90%
            cyc 100

            rem file: '5.7.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> [flying]>>. %0.80;0.90%
            cyc 100

            rem file: '5.8.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %0.70;0.90%
            nse $0.80;0.80;0.95$ <<robin --> [flying]> ==> <robin --> animal>>. %1.00;0.90%
            cyc 100

            rem file: '5.9.nal'
            nse $0.80;0.80;0.95$ <<robin --> bird> ==> <robin --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<robin --> bird> <=> <robin --> [flying]>>. %0.80;0.90%
            cyc 100

            rem file: '6.0.nal'
            nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> flyer>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<$y --> bird> ==> <$y --> flyer>>. %0.00;0.70%
            cyc 100

            rem file: '6.1.nal'
            nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<$y --> robin> ==> <$y --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '6.10.nal'
            nse $0.80;0.80;0.95$ (&&,<#x --> bird>,<#x --> swimmer>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <swan --> bird>. %0.90;0.90%
            cyc 100

            rem file: '6.11.nal'
            nse $0.80;0.80;0.95$ <{Tweety} --> [with_wings]>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(&&,<$x --> [chirping]>,<$x --> [with_wings]>) ==> <$x --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '6.12.nal'
            nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>, <(*, $x, worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%

            nse $0.80;0.80;0.95$ <{Tweety} --> flyer>. %1.00;0.90%
            cyc 100

            rem file: '6.13.nal'
            nse $0.80;0.80;0.95$ <(&&,<$x --> key>,<$y --> lock>) ==> <$y --> (/,open,$x,_)>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.14.nal'
            nse $0.80;0.80;0.95$ <<$x --> lock> ==> (&&,<#y --> key>,<$x --> (/,open,#y,_)>)>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.15.nal'
            nse $0.80;0.80;0.95$ (&&,<#x --> lock>,<<$y --> key> ==> <#x --> (/,open,$y,_)>>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.16.nal'
            nse $0.80;0.80;0.95$ (&&,<#x --> (/,open,#y,_)>,<#x --> lock>,<#y --> key>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.17.nal'
            nse $0.80;0.80;0.95$ <swan --> bird>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.80;0.90%
            cyc 100

            rem file: '6.18.nal'
            nse $0.80;0.80;0.95$ <gull --> swimmer>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <swan --> swimmer>. %0.80;0.90%
            cyc 100

            rem file: '6.19.nal'
            nse $0.80;0.80;0.95$ <{key1} --> (/,open,_,{lock1})>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%
            cyc 100

            rem file: '6.2.nal'
            nse $0.80;0.80;0.95$ <<$x --> swan> ==> <$x --> bird>>. %1.00;0.80%
            nse $0.80;0.80;0.95$ <<$y --> swan> ==> <$y --> swimmer>>. %0.80;0.90%
            cyc 100

            rem file: '6.20.nal'
            nse $0.80;0.80;0.95$ <<$x --> key> ==> <{lock1} --> (/,open,$x,_)>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.21.nal'
            nse $0.80;0.80;0.95$ (&&,<#x --> key>,<{lock1} --> (/,open,#x,_)>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <{lock1} --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.22.nal'
            nse $0.80;0.80;0.95$ <0 --> num>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<$1 --> num> ==> <(*,$1) --> num>>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <(*,(*,(*,0))) --> num>?
            cyc 100

            rem file: '6.23.nal'
            nse $0.80;0.80;0.95$ (&&,<#1 --> lock>,<<$2 --> key> ==> <#1 --> (/,open,$2,_)>>). %1.00;0.90%
            nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%
            cyc 100

            rem file: '6.24.nal'
            nse $0.80;0.80;0.95$ <<$1 --> lock> ==> (&&,<#2 --> key>,<$1 --> (/,open,#2,_)>)>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <{key1} --> key>. %1.00;0.90%
            cyc 100

            rem file: '6.25.nal'
            nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <lock1 --> lock>. %1.00;0.90%
            cyc 100

            rem file: '6.26.nal'
            nse $0.80;0.80;0.95$ <lock1 --> lock>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>. %1.00;0.90%
            cyc 100

            rem file: '6.27.nal'
            nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>. %1.00;0.90%
            cyc 100

            rem file: '6.3.nal'
            nse $0.80;0.80;0.95$ <<bird --> $x> ==> <robin --> $x>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<swimmer --> $y> ==> <robin --> $y>>. %0.70;0.90%
            cyc 100

            rem file: '6.4.nal'
            nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>) ==> <$x --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<$y --> [with_wings]> ==> <$y --> flyer>>. %1.00;0.90%
            cyc 100

            rem file: '6.5.nal'
            nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<$x --> [chirping]>, <(*, $x, worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%

            nse $0.80;0.80;0.95$ <(&&,<$x --> [chirping]>,<$x --> [with_wings]>) ==> <$x --> bird>>. %1.00;0.90%
            cyc 100

            rem file: '6.6.nal'
            nse $0.80;0.80;0.95$ <(&&,<$x --> flyer>,<(*,$x,worms) --> food>) ==> <$x --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<$y --> flyer> ==> <$y --> [with_wings]>>. %1.00;0.90%
            cyc 100

            rem file: '6.7.nal'
            nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            cyc 100

            rem file: '6.8.nal'
            nse $0.80;0.80;0.95$ <<$x --> bird> ==> <$x --> animal>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <tiger --> animal>. %1.00;0.90%
            cyc 100

            rem file: '6.9.nal'
            nse $0.80;0.80;0.95$ <<$x --> animal> <=> <$x --> bird>>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <robin --> bird>. %1.00;0.90%
            cyc 100

            rem file: '6.birdClaimedByBob.nal'
            nse $0.80;0.80;0.95$ <(&,<{Tweety} --> bird>,<bird --> fly>) --> claimedByBob>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <<(&,<#1 --> $2>,<$3 --> #1>) --> claimedByBob> ==> <<$3 --> $2> --> claimedByBob>>. %1.00;0.90%

            nse $0.90;0.80;1.00$ <?x --> claimedByBob>?
            cyc 100

            rem file: '6.can_of_worms.nal'
            nse $0.80;0.80;0.95$ <0 --> num>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <0 --> (/,num,_)>. %1.00;0.90%
            cyc 100

            rem file: '6.nlp1.nal'
            nse $0.80;0.80;0.95$ <(\\,REPRESENT,_,CAT) --> cat>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(\\,(\\,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish) --> cat>. %1.00;0.90%
            cyc 100

            rem file: '6.nlp2.nal'
            nse $0.80;0.80;0.95$ <cat --> (/,(/,REPRESENT,_,<(*,CAT,FISH) --> FOOD>),_,eat,fish)>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <cat --> CAT>. %1.00;0.90%
            cyc 100

            rem file: '6.redundant.nal'
            nse $0.80;0.80;0.95$ <<lock1 --> (/,open,$1,_)> ==> <$1 --> key>>. %1.00;0.90%
            cyc 100

            rem file: '6.symmetry.nal'
            nse $0.80;0.80;0.95$ <(*,a,b) --> like>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <(*,b,a) --> like>. %1.00;0.90%
            nse $0.90;0.80;1.00$ <<(*,$1,$2) --> like> <=> <(*,$2,$1) --> like>>?
            cyc 100

            rem file: '6.uncle.nal'
            nse $0.80;0.80;0.95$ <tim --> (/,uncle,_,tom)>. %1.00;0.90%
            nse $0.80;0.80;0.95$ <tim --> (/,uncle,tom,_)>. %0.00;0.90%
            cyc 100"#,
        )
    }
}
