//! `vnars.provider/1` entry point for NARust-158.

use anyhow::Result;
use narust_158::{
    inference::{match_task_and_belief, process_direct, reason, transform_task, InferenceEngine},
    parameters::DEFAULT_PARAMETERS,
    vm::alpha::LauncherAlpha,
    vnars_provider::serve,
};
use navm::vm::VmLauncher;
use std::io::{self, BufReader, BufWriter};

fn create_runtime() -> Result<impl navm::vm::VmRuntime> {
    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        match_task_and_belief,
        reason,
    );
    LauncherAlpha::new("vnars-provider", DEFAULT_PARAMETERS, ENGINE).launch()
}

fn main() -> Result<()> {
    let mut runtime = create_runtime()?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());
    serve(&mut runtime, input, &mut output)
}
