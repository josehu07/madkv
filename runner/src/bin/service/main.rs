//! Service launcher utility.

use color_print::cprintln;

use clap::Parser;

use runner::{RunnerError, ServerProc};

/// Launcher utility arguments.
#[derive(Parser, Debug)]
struct Args {
    /// Node ID I'm running on in the cluster. The format is as follows:
    ///   - Manager node is named "m". If there is replication, then the
    ///     replica ID follows after a '.', for example, "m.0", "m.1", etc.
    ///   - Server node starts with "s" followed by the partition ID, for
    ///     example, "s0", "s1", etc. If there is replication, then the
    ///     replica ID follows after a '.', for example, "s0.0", "s0.1",
    ///     "s0.2", "s1.0", etc.
    #[arg(short, long, default_value = "s0")]
    node_id: String,

    /// Server `just` invocation arguments, if not "none".
    #[arg(short, long, num_args(1..))]
    server_just_args: Vec<String>,

    /// Manager `just` invocation arguments, if not "none".
    #[arg(short, long, num_args(1..))]
    manager_just_args: Vec<String>,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();
    cprintln!("<s><yellow>Service launch configuration:</></> {:#?}", args);

    if args.node_id.is_empty() || (!args.node_id.starts_with('s') && !args.node_id.starts_with('m'))
    {
        return Err(RunnerError::Parse(format!(
            "invalid 'node_id' argument: {}",
            args.node_id
        )));
    }

    if args.node_id.starts_with('m') && args.manager_just_args[0].to_lowercase() != "none" {
        // we are launching a manager
        cprintln!("<s><yellow>Starting manager {}...</></>", args.node_id);
        let manager = ServerProc::new(args.manager_just_args.iter().map(|s| s.as_str()).collect())?;
        println!("  Launched...");
        manager.wait()?;
    } else if args.node_id.starts_with('s') && args.server_just_args[0].to_lowercase() != "none" {
        // we are launching a server
        cprintln!("<s><yellow>Starting server {}...</></>", args.node_id);
        let server = ServerProc::new(args.server_just_args.iter().map(|s| s.as_str()).collect())?;
        println!("  Launched...");
        server.wait()?;
    }

    Ok(())
}
