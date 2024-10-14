use crate::{
    trash::{TrashItem, TrashManager},
    xattr::ExtendedAttributes,
    Result,
};
use clap::Args;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};

#[derive(Args)]
pub struct ListArgs {
    /// Filter by original path substring.
    #[clap(short, long)]
    pub filter_path: Option<String>,
}

pub fn handle_list<T: ExtendedAttributes>(
    trash_manager: TrashManager<T>,
    args: ListArgs,
) -> Result<()> {
    // Get all entries in the trash and filter them
    let mut items: Vec<TrashItem> = trash_manager
        .list_items()?
        .into_iter()
        .filter(|entry| {
            // TODO: Implement date filtering
            if let Some(path) = args.filter_path.as_ref() {
                if !entry.original_path.contains(path) {
                    return false;
                }
            }
            true
        })
        .collect();

    match (args.filter_path.as_ref(), items.is_empty()) {
        (Some(filter_path), false) => {
            println!(
                "Items in the trash matching the path filter: '{}'",
                filter_path
            );
        }
        (Some(filter_path), true) => {
            println!(
                "No items found in the trash matching the path filter: '{}'",
                filter_path
            );
        }
        (None, true) => {
            println!("The trash is empty.");
            return Ok(());
        }
        _ => {}
    }

    // Sort by deletion date
    items.sort_by_key(|entry| entry.deletion_date);

    // Print the items in a table
    let mut table = Table::new();
    table
        .set_header(vec!["Original Path", "ID", "Kind", "Deletion Date"])
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    for item in items {
        let deletion_date_display = item.format_deletion_date();
        let kind = item.kind().to_string();
        table.add_row(vec![
            item.original_path,
            item.id,
            kind,
            deletion_date_display,
        ]);
    }

    println!("{}", table);
    Ok(())
}
