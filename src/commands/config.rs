use crate::{config::Config, xattr::ExtendedAttributes, Result};
use clap::{Args, Subcommand, ValueEnum};

#[derive(Args)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    subcommand: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    #[clap(about = "Get the value of a configuration key")]
    Get {
        #[clap(
            short,
            long,
            help = "The name of the configuration key to get.",
            required = true
        )]
        key: ConfigKey,
    },

    #[clap(about = "Set the value of a configuration key")]
    Set {
        #[clap(
            short,
            long,
            help = "The name of the configuration key to set.",
            required = true
        )]
        key: ConfigKey,
        #[clap(
            short,
            long,
            help = "The value to set the configuration key to.",
            required = true
        )]
        value: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ConfigKey {
    #[clap(help = "The directory where deleted files are moved.")]
    TrashDir,
    #[clap(help = "The number of days to wait before deleting the item permanently.")]
    GracePeriod,
}

pub fn handle_config<T: ExtendedAttributes>(config: Config<T>, args: ConfigArgs) -> Result<()> {
    match args.subcommand {
        ConfigAction::Get { key } => match key {
            ConfigKey::TrashDir => println!("Trash directory: {}", config.trash_dir.display()),
            ConfigKey::GracePeriod => {
                println!("Grace period in days: {}", config.grace_period_in_days)
            }
        },
        ConfigAction::Set { key, value } => match key {
            ConfigKey::TrashDir => {
                config.set_trash_dir(value.as_str())?;
                println!("Set trash directory to {}", value);
            }
            ConfigKey::GracePeriod => match value.parse::<u32>() {
                Ok(value) => {
                    config.set_grace_period(value)?;
                    println!("Set grace period in days to {}", value);
                }
                Err(_) => eprintln!("Grace period must be a positive integer."),
            },
        },
    }

    Ok(())
}
