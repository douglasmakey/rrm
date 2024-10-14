use crate::{xattr::ExtendedAttributes, Error, Result};
use std::{env, path::PathBuf};

// Constants used to store the trash directory path and grace period in the extended attributes.
const TRASH_DIR_ATTR: &str = "trash_dir";
const GRACE_PERIOD_ATTR: &str = "grace_period_in_days";

/// Name of the default directory used to store trashed items in the user's home directory.
const TRASH_DIR_NAME: &str = concat!(env!("HOME"), "/.tmp_trash");

/// Default grace period in days before permanently deleting trashed items.
const DEFAULT_GRACE_PERIOD_IN_DAYS: u32 = 7;

#[derive(Debug)]
pub struct Config<T: ExtendedAttributes> {
    pub grace_period_in_days: u32,
    pub trash_dir: PathBuf,
    xattr_manager: T,
    bin_path: PathBuf,
}

impl<T: ExtendedAttributes> Config<T> {
    pub fn load(xattr_manager: T) -> Result<Self> {
        // Get the path to the binarys
        let bin = env::current_exe()?
            .to_str()
            .ok_or_else(|| {
                Error::InvalidBinaryPath("Failed to convert binary path to string".to_string())
            })?
            .to_string();

        let bin_path = PathBuf::from(&bin);
        let trash_path = match xattr_manager.get_attr(&bin_path, TRASH_DIR_ATTR)? {
            // If the value is not empty, use it as the trash directory path.
            Some(val) if !val.is_empty() => val,
            _ => TRASH_DIR_NAME.to_string(),
        };

        let trash_dir = ensure_trash_folder(&trash_path)?;
        let grace_period_in_days: u32 =
            match xattr_manager.get_attr(&trash_dir, GRACE_PERIOD_ATTR)? {
                // If the value is not a valid number (empty is included), use the default grace period.
                Some(val) => val.parse().unwrap_or(DEFAULT_GRACE_PERIOD_IN_DAYS),
                None => DEFAULT_GRACE_PERIOD_IN_DAYS,
            };

        Ok(Self {
            trash_dir,
            grace_period_in_days,
            bin_path,
            xattr_manager,
        })
    }

    /// Sets the grace period (in days) before permanently deleting items.
    /// The grace period is stored in the extended attributes of the trash folder.
    ///
    /// # Arguments
    ///
    /// * `days` - The number of days to wait before deleting the item permanently.
    pub fn set_grace_period(&self, days: u32) -> Result<()> {
        self.xattr_manager
            .set_attr(&self.trash_dir, GRACE_PERIOD_ATTR, &days.to_string())
    }

    /// Sets the directory where trashed items are stored.
    /// The trash directory path is stored in the binary's extended attributes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory where trashed items should be stored.
    pub fn set_trash_dir(&self, path: &str) -> Result<()> {
        // let trash_xattr = xattr::XAttrManager::new(&self.bin_path);
        self.xattr_manager
            .set_attr(&self.bin_path, TRASH_DIR_ATTR, path)
    }
}

fn ensure_trash_folder(path: &str) -> Result<PathBuf> {
    let trash_dir = PathBuf::from(path);
    if !trash_dir.exists() {
        std::fs::create_dir(&trash_dir)?;
    }
    Ok(trash_dir)
}
