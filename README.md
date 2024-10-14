# rrm

Introducing `rrm`: The command-line tool nobody asked for and nobody knew they wanted—but here it is anyway!

Ever had that sinking feeling after accidentally nuking an important file with rm? Yeah, I've been there too. Sure, other tools might already solve this problem, but who needs those when you can have `rrm`, built with love (and a bit of YOLO) in Rust?

`rrm` adds a sprinkle of safety to your command-line adventures by moving your precious (or not-so-precious) files to a safe trash bin instead of obliterating them instantly. With a customizable grace period, you've got time to realize, ***"Oops, I actually needed that!"***

Why did I make it? Because I could! It's an opportunity to play with Rust and keep things fun and quirky. So go ahead:
- Delete with confidence—knowing you can restore if needed.
- Embrace the grace (period)—set it to your liking.
- Live on the edge—but with a safety net.

Remember, life’s too short for accidental deletions. Give `rrm` a whirl, and make your command-line experience not just safer, but a tad more entertaining!

> [!NOTE]
> If anyone asks why you're using it, just say it's for the fun and the love of Rust. After all, who doesn't enjoy a side quest in their coding journey?


Features
- Safe Deletion: Moves files and directories to a trash directory instead of deleting them immediately.
- Restore Capability: Restore trashed items back to their original locations.
- Grace Period: Set a grace period after which trashed items are permanently deleted.
- Listing Trash: View the contents of the trash directory with optional filters.
- Cleaning: Clean up items that have passed their grace period or delete all items immediately.
- Configuration Management: View and edit the tool's configuration settings.
- Extended Attributes: Stores original file paths and deletion dates using extended file attributes for accurate restoration and management.

## Installation

To install rrm, ensure you have Rust installed on your system. Then, clone the repository and build the project:

```bash
$ git clone https://github.com/douglasmakey/rrm.git
$ cd rrm
$ cargo build --release

# Optionally, install the binary to your Cargo bin directory
$ cargo install --path . 
```

Or copy the executable to a directory in your PATH:

```bash
$ cp target/release/rrm /usr/local/bin/
```

## Usage

The general syntax for using rrm is:

```bash
$ rrm <COMMAND> [OPTIONS]
```

### Commands

#### Command: `rm`

Description: Remove files or directories by moving them to the trash directory.

**Usage:**

``` bash
$ rrm rm [OPTIONS] <FILES>...
```

**Options:**

- `-i`, `--immediate`: Immediate remove without moving to trash.
- `-a`, `--auto-clean`: Automatically clean files that have passed the grace period.
- `-g`, `--grace-period-in-days` <DAYS>: Set the number of days to wait before deleting the file permanently.

**Examples:**

```bash
# Move a file to the trash
$ rrm rm file.txt

# Move multiple files to the trash
$ rrm rm file1.txt file2.txt

# Immediate delete a file without moving to trash
$ rrm rm -i file.txt

# Set a custom grace period
$ rrm rm --grace-period-in-days 7 file.txt

# Move a file to the trash and automatically clean files that have passed the grace period
$ rrm rm file.txt --auto-clean
```

#### Command: `restore`

Description: Restore a file or directory from the trash back to its original location.

**Usage:**

```bash
$ rrm restore <ID>
```

**Options:**

- ID: The unique identifier of the trashed item (as shown in the list command).
- `-r`, `--rename` [STRING]: Rename the item to the specified name after restoring it.

**Example:**

```bash
# Restore a trashed file using its ID
$ rrm restore 123e4567-e89b-12d3-a456-426614174000

# Restore a trashed file to its original location with a new name, which is useful when the original path already exists
$ rrm restore 123e4567-e89b-12d3-a456-426614174000 -r new_name
```

#### Command: `list`

Description: List files and directories currently in the trash.

**Usage:**

```bash
$ rrm list [OPTIONS]
```

**Options:**

- `-f`, `--filter-path` [STRING]: Filter trashed items by original path substring.

**Example:**

```bash
# List all trashed items
$ rrm list

╭──────────────────────────────────────────────────────────────┬──────────────────────────────────────┬──────┬─────────────────────╮
│ Original Path                                                ┆ ID                                   ┆ Kind ┆ Deletion Date       │
╞══════════════════════════════════════════════════════════════╪══════════════════════════════════════╪══════╪═════════════════════╡
│ /Users/douglasmakey/workdir/personal/rust-learning/rrm/a.txt ┆ 784205c5-294a-434f-a50d-03314d5f72e5 ┆ File ┆ 2024-10-21 05:06:39 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ /Users/douglasmakey/workdir/personal/rust-learning/rrm/b.txt ┆ 2326c23d-1720-4719-a7c2-1201b6bb63cb ┆ File ┆ 2024-10-21 05:06:42 │
╰──────────────────────────────────────────────────────────────┴──────────────────────────────────────┴──────┴─────────────────────╯

# List trashed items that contain 'project' in their original path
$ rrm list --filter-path project
```

#### Command: `clean`

Description: Clean files and directories that have passed the grace period or immediately delete all trashed items.

**Usage:**

```bash
$ rrm clean [OPTIONS]
```

**Options:**

- `-i`, `--immediate`: Immediately clean all items in the trash, regardless of their grace period.

**Examples:**

```bash
# Clean items that have passed the grace period
$ rrm clean

# Immediately clean all items in the trash
$ rrm clean --immediate
```

#### Command: `config`

Description: Show or edit the configuration settings for rrm.

**Usage:**

```bash
$ rrm config [<COMMAND>]
```

**Examples:**

```bash
# Show current configuration
$ rrm config

# Set the default grace period to 10 days
$ rrm config set grace_period_in_days 10

# Get the current grace period setting
$ rrm config get grace_period_in_days
```

### Global Options
- `-h`, `--help`: Show help information.
- `-v`, `-vv`, `-vvv` : Set verbose

## Configuration

The configuration allows you to customize the behavior of `rrm`. The primary configuration options include:

- **Trash Directory**: The directory where trashed items are stored. By default, this is set to `$HOME/tmp_trash`.
- **Grace Period**: The number of days before trashed items are permanently deleted. The default grace period is 7 days.

These values are stored using extended attributes: `trashdir` in the `rrm` binary and `grace_period` in the `trash_dir`.













