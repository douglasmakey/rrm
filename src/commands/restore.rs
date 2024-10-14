use crate::{trash::TrashManager, xattr::ExtendedAttributes, Result};
use clap::Args;

#[derive(Args)]
pub struct RestoreArgs {
    #[clap(help = "The ID of the file or directory to restore.", required = true)]
    pub id: String,
    #[clap(
        short,
        long,
        help = "Rename the item to the specified name after restoring it."
    )]
    pub rename: Option<String>,
}

pub fn handle_restore<T: ExtendedAttributes>(
    trash_manager: TrashManager<T>,
    args: RestoreArgs,
) -> Result<()> {
    trash_manager.restore_item_by_id(&args.id, args.rename)
}
