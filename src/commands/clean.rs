use crate::{trash::TrashManager, xattr::ExtendedAttributes, Result};
use clap::Args;

#[derive(Args)]
pub struct CleanArgs {
    #[clap(
        short,
        long,
        help = "Immediately delete all files or directories in the trash, even if they have not passed the grace period",
        default_value = "false"
    )]
    pub immediate: bool,
}

pub fn handle_clean<T: ExtendedAttributes>(
    trash_manager: TrashManager<T>,
    args: CleanArgs,
) -> Result<()> {
    trash_manager.clean_trash(args.immediate)
}
