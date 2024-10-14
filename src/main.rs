mod commands;
mod config;
mod error;
mod trash;
mod xattr;

use clap::{ArgAction, Parser, Subcommand};
use commands::{
    clean::{handle_clean, CleanArgs},
    config::{handle_config, ConfigArgs},
    list::{handle_list, ListArgs},
    restore::{handle_restore, RestoreArgs},
    rm::{handle_rm, RmArgs},
};
pub use error::{Error, Result};
use xattr::XAttrManager;

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Remove files or directories")]
    Rm(RmArgs),

    #[clap(about = "Restore a file or directory from the trash")]
    Restore(RestoreArgs),

    #[clap(about = "List files and directories in the trash")]
    List(ListArgs),

    #[clap(about = "Clean files and directories that have passed the grace period")]
    Clean(CleanArgs),

    #[clap(about = "Show or edit the configuration")]
    Config(ConfigArgs),
}

#[derive(Parser)]
struct App {
    #[clap(subcommand)]
    cmd: Commands,
    #[clap(
        long,
        short,
        action = ArgAction::Count,
        help = "Increase verbosity level (use multiple times for more verbosity)"
    )]
    verbose: u8,
}

fn main() {
    run().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}

fn run() -> Result<()> {
    let app = App::parse();

    // Set the log level based on the verbosity flag
    let log_level = match app.verbose {
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        4 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Off,
    };

    env_logger::Builder::from_default_env()
        .filter(None, log_level)
        .format_timestamp(None)
        .init();

    let xattr_manager = XAttrManager::new()?;
    let config = config::Config::load(xattr_manager)?;
    let trash_manager = trash::TrashManager::new(config.trash_dir.clone(), xattr_manager);

    match app.cmd {
        Commands::Rm(args) => handle_rm(config, trash_manager, args),
        Commands::List(args) => handle_list(trash_manager, args),
        Commands::Restore(args) => handle_restore(trash_manager, args),
        Commands::Clean(args) => handle_clean(trash_manager, args),
        Commands::Config(args) => handle_config(config, args),
    }?;

    Ok(())
}
