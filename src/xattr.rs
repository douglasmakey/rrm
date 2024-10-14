use crate::Result;
use derive_more::derive::Display;
use std::{
    io::{self},
    path::{Path, PathBuf},
};

/// Namespace for extended attributes (xattrs) on macOS and other operating systems.
/// On macOS, this is an empty string, while on other operating systems, it is "user.".
#[cfg(target_os = "macos")]
const XATTR_NAMESPACE: &str = "";
#[cfg(not(target_os = "macos"))]
const XATTR_NAMESPACE: &str = "user.";

#[derive(Debug, Display)]
pub enum XAttrError {
    #[display("Extended attributes are not supported on this platform")]
    UnsupportedPlatform,
    #[display("Failed to set attribute '{}' on '{}': {}", attr, path.display(), source)]
    SetAttr {
        attr: String,
        path: PathBuf,
        source: io::Error,
    },

    #[display("Failed to get attribute '{}' from '{}': {}", attr, path.display(), source)]
    GetAttr {
        attr: String,
        path: PathBuf,
        source: io::Error,
    },

    #[display("Failed to remove attribute '{}' from '{}': {}", attr, path.display(), source)]
    RemoveAttr {
        attr: String,
        path: PathBuf,
        source: io::Error,
    },

    #[display("Failed to set attribute '{}' on '{}': {}", attr, path.display(), source)]
    InvalidUtf8 {
        attr: String,
        path: PathBuf,
        source: std::string::FromUtf8Error,
    },
}

pub trait ExtendedAttributes {
    fn set_attr(&self, path: &Path, key: &str, value: &str) -> Result<()>;
    fn get_attr(&self, path: &Path, key: &str) -> Result<Option<String>>;
    fn remove_attr(&self, path: &Path, key: &str) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct XAttrManager {}

impl XAttrManager {
    pub fn new() -> Result<Self> {
        if !xattr::SUPPORTED_PLATFORM {
            return Err(XAttrError::UnsupportedPlatform)?;
        }

        Ok(Self {})
    }
}

impl ExtendedAttributes for XAttrManager {
    /// Sets an extended attribute on the file or directory.
    fn set_attr(&self, path: &Path, attr: &str, value: &str) -> Result<()> {
        let attr_name = format!("{}{}", XATTR_NAMESPACE, attr);
        Ok(
            xattr::set(path, &attr_name, value.as_bytes()).map_err(|e| XAttrError::SetAttr {
                attr: attr_name,
                path: path.to_path_buf(),
                source: e,
            })?,
        )
    }

    /// Removes an extended attribute from the file or directory.
    fn remove_attr(&self, path: &Path, attr: &str) -> Result<()> {
        let attr_name = format!("{}{}", XATTR_NAMESPACE, attr);
        Ok(
            xattr::remove(path, &attr_name).map_err(|e| XAttrError::RemoveAttr {
                attr: attr_name,
                path: path.to_path_buf(),
                source: e,
            })?,
        )
    }

    /// Retrieves an extended attribute from the file or directory.
    fn get_attr(&self, path: &Path, attr: &str) -> Result<Option<String>> {
        let attr_name = format!("{}{}", XATTR_NAMESPACE, attr);
        match xattr::get(path, &attr_name) {
            Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| {
                XAttrError::InvalidUtf8 {
                    attr: attr_name,
                    path: path.to_path_buf(),
                    source: e,
                }
            })?)),
            Ok(None) => Ok(None),
            Err(e) => Err(XAttrError::GetAttr {
                attr: attr_name,
                path: path.to_path_buf(),
                source: e,
            })?,
        }
    }
}
