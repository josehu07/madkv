//! Service launcher utility.

use color_print::cprintln;

use clap::Parser;

use runner::{RunnerError, ServerProc};

/// Launcher utility arguments.
#[derive(Parser, Debug)]
struct Args {
    /// Server `just` invocation arguments.
    #[arg(short, long, num_args(1..))]
    just_args: Vec<String>,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();
    cprintln!("<s><yellow>Service launch configuration:</></> {:#?}", args);

    let server = ServerProc::new(args.just_args.iter().map(|s| s.as_str()).collect())?;

    server.wait()?;
    Ok(())
}
