use crate::{config::Config, trash::TrashManager, xattr::ExtendedAttributes, Result};
use clap::Args;
use log::info;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Args)]
pub struct RmArgs {
    /// Files or directories to remove.
    #[clap(required = true)]
    pub paths: Vec<String>,
    #[clap(
        short,
        long,
        help = "Inmediately delete files or directories without moving them to the trash",
        default_value = "false"
    )]
    pub immediate: bool,
    #[clap(
        short,
        long,
        help = "Automatically clean files and directories that have passed the grace period",
        default_value = "false"
    )]
    pub auto_clean: bool,
    /// The number of days to wait before deleting the item permanently.
    #[clap(
        short,
        long,
        help = "The number of days to wait before deleting the files or directories permanently"
    )]
    pub grace_period_in_days: Option<u32>,
}

pub fn handle_rm<T: ExtendedAttributes>(
    config: Config<T>,
    trash_manager: TrashManager<T>,
    args: RmArgs,
) -> Result<()> {
    if args.immediate {
        return delete_paths(args.paths);
    }

    let now = chrono::Utc::now();
    let grace_period_in_days = args
        .grace_period_in_days
        .map(|d| d as i64)
        .unwrap_or(config.grace_period_in_days as i64);

    let deletion_date = now
        .checked_add_signed(chrono::Duration::days(grace_period_in_days))
        .expect("Failed to add grace period to current time");

    let paths: Vec<PathBuf> = args
        .paths
        .iter()
        .filter_map(|p| {
            let path = PathBuf::from(p);
            if path.exists() {
                Some(path)
            } else {
                eprintln!("{}: No such file or directory", path.display());
                None
            }
        })
        .collect();

    trash_manager.trash_items(paths, deletion_date)?;
    if args.auto_clean {
        info!(
            "Automatically cleaning trash..items that have passed the grace period will be deleted"
        );
        trash_manager.clean_trash(false)?;
    }

    Ok(())
}

fn delete_paths(paths: Vec<String>) -> Result<()> {
    for path in paths {
        let path = Path::new(&path);
        if !path.exists() {
            eprintln!("{}: No such file or directory", path.display());
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
