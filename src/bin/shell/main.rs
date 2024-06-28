use anyhow::Result;
use narust_158::{control::DEFAULT_PARAMETERS, inference::InferenceEngine, vm::Launcher};
use navm::vm::{VmLauncher, VmRuntime};

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

fn shell(mut runtime: impl VmRuntime) -> Result<()> {
    let mut input = String::new();
    let stdin = &std::io::stdin();
    loop {
        // in
        let bytes = stdin.read_line(&mut input)?;
        if bytes == 0 {
            continue;
        }
        match navm::cmd::Cmd::parse(input.trim()) {
            Ok(cmd) => runtime.input_cmd(cmd)?,
            Err(err) => eprintln!("NAVM cmd parse error: {err}"),
        }
        // out
        while let Some(output) = runtime.try_fetch_output()? {
            println!("{}", output.to_json_string());
        }
        // clear
        input.clear()
    }
}

pub fn main() -> Result<()> {
    let runtime = create_runtime()?;
    shell(runtime)?;
    Ok(())
}
