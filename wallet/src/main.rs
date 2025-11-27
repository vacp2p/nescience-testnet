use anyhow::Result;
use clap::{CommandFactory, Parser};
use tokio::runtime::Builder;
use wallet::{Args, OverCommand, execute_continious_run, execute_setup, execute_subcommand};

pub const NUM_THREADS: usize = 2;

// TODO #169: We have sample configs for sequencer, but not for wallet
// TODO #168: Why it requires config as a directory? Maybe better to deduce directory from config
// file path? TODO #172: Why it requires config as env var while sequencer_runner accepts as
// argument? TODO #171: Running pinata doesn't give output about transaction hash and etc.
fn main() -> Result<()> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(NUM_THREADS)
        .enable_all()
        .build()
        .unwrap();

    let args = Args::parse();

    env_logger::init();

    runtime.block_on(async move {
        if let Some(overcommand) = args.command {
            match overcommand {
                OverCommand::Command(command) => {
                    execute_subcommand(command).await.unwrap();
                }
                OverCommand::Setup { password } => {
                    execute_setup(password).await.unwrap();
                }
            }
        } else if args.continious_run {
            execute_continious_run().await.unwrap();
        } else {
            let help = Args::command().render_long_help();
            println!("{help}");
        }
    });

    Ok(())
}
