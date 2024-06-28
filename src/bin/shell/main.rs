use anyhow::Result;
use narust_158::{
    control::DEFAULT_PARAMETERS,
    inference::InferenceEngine,
    vm::{Launcher, Runtime},
};
use navm::vm::{VmLauncher, VmRuntime};

fn create_runtime() -> Result<Runtime> {
    let vm = Launcher::new("nar_158", DEFAULT_PARAMETERS, InferenceEngine::VOID);
    vm.launch()
}

fn shell(mut runtime: Runtime) -> Result<()> {
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
