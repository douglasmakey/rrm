use crate::{xattr::ExtendedAttributes, Error, Result};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use std::{fs, path::PathBuf};
use uuid::Uuid;

const ORIGINAL_PATH_ATTR: &str = "original_path";
const DELETION_DATE_ATTR: &str = "deletion_date";

pub struct TrashItem {
    pub id: String,
    pub path: PathBuf,
    pub original_path: String,
    pub deletion_date: DateTime<Utc>,
}

impl TrashItem {
    pub fn kind(&self) -> &str {
        if self.path.is_dir() {
            "Directory"
        } else {
            "File"
        }
    }

    /// Formats the deletion date for display purposes.
    pub fn format_deletion_date(&self) -> String {
        let now = Utc::now().date_naive();
        let deletion_date = self.deletion_date.date_naive();

        match deletion_date {
            d if d == now => "Today".to_string(),
            d if d == now.succ_opt().unwrap_or(now) => "Tomorrow".to_string(),
            _ => self.deletion_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

pub struct TrashManager<T: ExtendedAttributes> {
    trash_dir: PathBuf,
    xattr_manager: T,
}

impl<T: ExtendedAttributes> TrashManager<T> {
    pub fn new(trash_dir: PathBuf, xattr_manager: T) -> Self {
        Self {
            trash_dir,
            xattr_manager,
        }
    }

    /// Moves the specified items to the trash.
    pub fn trash_items(&self, paths: Vec<PathBuf>, deletion_date: DateTime<Utc>) -> Result<()> {
        for path in paths {
            // Gets the original path"
            let original_path = path.canonicalize()?;
            let original_path_str = match original_path.to_str() {
                Some(p) => p,
                None => {
                    error!(
                        "Failed to convert original path to string: {} cannot be represented as UTF-8",
                        original_path.display()
                    );
                    continue;
                }
            };

            // Sets extended attributes on the trashed item
            self.xattr_manager
                .set_attr(&path, ORIGINAL_PATH_ATTR, original_path_str)?;
            self.xattr_manager
                .set_attr(&path, DELETION_DATE_ATTR, &deletion_date.to_rfc3339())?;

            // Generate a unique id to prevent collisions
            let unique_id = Uuid::new_v4().to_string();
            let trashed_item_path = self.trash_dir.join(&unique_id);

            // Move the item to the trash directory
            fs::rename(path, &trashed_item_path)?;
        }
        Ok(())
    }

    /// Retrieves a list of items currently in the trash.
    pub fn list_items(&self) -> Result<Vec<TrashItem>> {
        let mut items: Vec<TrashItem> = Vec::new();
        for entry_result in self.trash_dir.read_dir()? {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    println!("Failed to read entry in trash directory: {}", e);

                    error!("Failed to read entry in trash directory: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            let id = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<Invalid UTF-8>")
                .to_string();

            // Get the extended attributes
            let original_path = match self.xattr_manager.get_attr(&path, ORIGINAL_PATH_ATTR) {
                Ok(Some(val)) => val,
                _ => {
                    warn!(
                        "Missing '{ORIGINAL_PATH_ATTR}' for item with id: '{}' - maybe it was not deleted by rrm?",
                        id
                    );
                    continue;
                }
            };

            let deletion_date_str = match self.xattr_manager.get_attr(&path, DELETION_DATE_ATTR) {
                Ok(Some(val)) => val,
                _ => {
                    warn!("Missing '{DELETION_DATE_ATTR}' for item with id: '{}' - maybe it was not deleted by rrm?", id);
                    continue;
                }
            };

            let deletion_date = match DateTime::parse_from_rfc3339(&deletion_date_str) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(_) => {
                    error!("Failed to parse deletion date for item with id: {}", id);
                    continue;
                }
            };

            items.push(TrashItem {
                id,
                path,
                original_path,
                deletion_date,
            });
        }

        Ok(items)
    }

    /// Restores an item from the trash by its ID.
    pub fn restore_item_by_id(&self, id: &str, rename: Option<String>) -> Result<()> {
        let item_path = self.trash_dir.join(id);
        if !item_path.exists() {
            return Err(Error::ItemNotFound(id.to_string()));
        }

        let original_path = self
            .xattr_manager
            .get_attr(&item_path, ORIGINAL_PATH_ATTR)?
            .ok_or_else(|| Error::MissingAttribute {
                attr: ORIGINAL_PATH_ATTR.to_string(),
                id: id.to_string(),
            })?;

        // Get the original path
        let mut original_path = PathBuf::from(original_path);

        // Rename the item if a new name is provided
        let original_path = if let Some(new_name) = rename {
            original_path.set_file_name(new_name);
            original_path
        } else {
            original_path
        };

        if original_path.exists() {
            return Err(Error::PathAlreadyExists(
                original_path.to_string_lossy().to_string(),
            ));
        }

        if let Some(parent) = original_path.parent() {
            if !parent.exists() {
                warn!(
                    "Parent directory of the original path does not exist: {}",
                    parent.display()
                );
                return Err(Error::InvalidOriginalPath(
                    original_path.to_string_lossy().to_string(),
                ));
            }
        }

        // Remove the xattr attributes
        self.xattr_manager
            .remove_attr(&item_path, ORIGINAL_PATH_ATTR)?;
        self.xattr_manager
            .remove_attr(&item_path, DELETION_DATE_ATTR)?;

        fs::rename(&item_path, &original_path)?;
        Ok(())
    }

    pub fn clean_trash(&self, immediate: bool) -> Result<()> {
        let items = self.list_items()?;
        info!("Trash items found: {}", items.len());
        let now = Utc::now();
        let mut items_deleted = 0;
        for item in items {
            if (immediate || item.deletion_date < now) && item.path.exists() {
                info!(
                    "Deleting item with id: {} and original path: {}",
                    item.id, item.original_path
                );
                self.delete_item_permanently(item)?;
                items_deleted += 1;
            }
        }

        info!("Items deleted from trash: {}", items_deleted);
        Ok(())
    }

    fn delete_item_permanently(&self, item: TrashItem) -> Result<()> {
        assert!(item.path.exists());
        if item.path.is_dir() {
            fs::remove_dir_all(&item.path)?;
        } else {
            fs::remove_file(&item.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::{mock, predicate::in_iter};
    use tempfile::{tempdir, NamedTempFile};

    mock! {
        pub XattrManager {}
        impl ExtendedAttributes for XattrManager {
            fn set_attr(&self, path: &std::path::Path, key: &str, value: &str) -> crate::Result<()>;
            fn get_attr(&self, path: &std::path::Path, key: &str) -> crate::Result<Option<String>>;
            fn remove_attr(&self, path: &std::path::Path, key: &str) -> crate::Result<()>;
        }
    }

    #[test]
    fn test_trash_items() -> Result<()> {
        let deletion_date = Utc::now();
        let temp_dir = tempdir()?;
        let trash_dir = temp_dir.path().to_path_buf();
        assert!(trash_dir.exists());

        // Create two temporary files
        // Create a temporary file
        let temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        assert!(original_path.exists());
        let original_path_canonicalized = original_path.canonicalize()?;
        let original_path_str = original_path_canonicalized.to_str().unwrap().to_string();

        // Create a second temporary file
        let temp_file2 = NamedTempFile::new()?;
        let original_path2 = temp_file2.path().to_path_buf();
        assert!(original_path2.exists());
        let original_path2_canonicalized = original_path2.canonicalize()?;
        let original_path2_str = original_path2_canonicalized.to_str().unwrap().to_string();

        let mut xattr_manager = MockXattrManager::new();
        xattr_manager
            .expect_set_attr()
            .with(
                in_iter(vec![original_path.clone(), original_path2.clone()]),
                in_iter(vec![ORIGINAL_PATH_ATTR, DELETION_DATE_ATTR]),
                in_iter(vec![
                    original_path_str,
                    original_path2_str,
                    deletion_date.to_rfc3339().to_string(),
                ]),
            )
            .times(4)
            .returning(|_, _, _| Ok(()));

        let trash_manager = TrashManager::new(trash_dir.clone(), xattr_manager);
        trash_manager.trash_items(vec![original_path, original_path2], deletion_date)?;

        // Check if the files were moved to the trash
        assert_eq!(trash_dir.read_dir()?.count(), 2);
        Ok(())
    }

    #[test]
    fn test_list_items() -> Result<()> {
        let deletion_date = Utc::now();
        let temp_dir = tempdir()?;
        let trash_dir = temp_dir.path().to_path_buf();
        assert!(trash_dir.exists());

        // Create two temporary files
        // Create a temporary file
        let temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        assert!(original_path.exists());
        let original_path_canonicalized = original_path.canonicalize()?;
        let original_path_str = original_path_canonicalized.to_str().unwrap().to_string();

        // Create a second temporary file
        let temp_file2 = NamedTempFile::new()?;
        let original_path2 = temp_file2.path().to_path_buf();
        assert!(original_path2.exists());
        let original_path2_canonicalized = original_path2.canonicalize()?;
        let original_path2_str = original_path2_canonicalized.to_str().unwrap().to_string();

        let mut xattr_manager = MockXattrManager::new();
        xattr_manager
            .expect_set_attr()
            .with(
                in_iter(vec![original_path.clone(), original_path2.clone()]),
                in_iter(vec![ORIGINAL_PATH_ATTR, DELETION_DATE_ATTR]),
                in_iter(vec![
                    original_path_str,
                    original_path2_str,
                    deletion_date.to_rfc3339().to_string(),
                ]),
            )
            .times(4)
            .returning(|_, _, _| Ok(()));

        xattr_manager
            .expect_get_attr()
            .times(4)
            .returning(move |_, key| match key {
                DELETION_DATE_ATTR => Ok(Some(deletion_date.to_rfc3339())),
                _ => Ok(Some("some_path".to_string())),
            });

        let trash_manager = TrashManager::new(trash_dir.clone(), xattr_manager);
        trash_manager.trash_items(vec![original_path, original_path2], deletion_date)?;

        let items = trash_manager.list_items()?;
        assert_eq!(trash_dir.read_dir()?.count(), 2);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].deletion_date, deletion_date);
        assert_eq!(items[1].deletion_date, deletion_date);
        Ok(())
    }

    #[test]
    fn clean_trash_delete_old_file() -> Result<()> {
        let deletion_date_past = Utc::now() - chrono::Duration::days(1);

        let temp_dir = tempdir()?;
        let trash_dir = temp_dir.path().to_path_buf();
        assert!(trash_dir.exists());

        // Create two temporary files
        // Create a temporary file
        let temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        assert!(original_path.exists());
        let original_path_canonicalized = original_path.canonicalize()?;
        let original_path_str = original_path_canonicalized.to_str().unwrap().to_string();

        let mut xattr_manager = MockXattrManager::new();
        xattr_manager
            .expect_set_attr()
            .with(
                in_iter(vec![original_path.clone()]),
                in_iter(vec![ORIGINAL_PATH_ATTR, DELETION_DATE_ATTR]),
                in_iter(vec![
                    original_path_str,
                    deletion_date_past.to_rfc3339().to_string(),
                ]),
            )
            .times(2)
            .returning(|_, _, _| Ok(()));

        xattr_manager
            .expect_get_attr()
            .times(2)
            .returning(move |_, key| match key {
                DELETION_DATE_ATTR => Ok(Some(deletion_date_past.to_rfc3339())),
                _ => Ok(Some("some_path".to_string())),
            });

        let trash_manager = TrashManager::new(trash_dir.clone(), xattr_manager);
        trash_manager.trash_items(vec![original_path], deletion_date_past)?;

        trash_manager.clean_trash(false)?;

        // Check if the files were moved to the trash
        let items = trash_manager.list_items()?;
        assert_eq!(trash_dir.read_dir()?.count(), 0);
        assert_eq!(items.len(), 0);
        Ok(())
    }
}
