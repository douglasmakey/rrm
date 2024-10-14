use derive_more::{Display, From};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display, From)]
pub enum Error {
    #[display("Failed to load binary path executable: {}", _0)]
    InvalidBinaryPath(String),

    #[display("The attribute '{}' is missing from the item '{}'", attr, id)]
    MissingAttribute { attr: String, id: String },

    #[display("Invalid original path: '{}'", _0)]
    InvalidOriginalPath(String),

    #[display("Path '{}' already exists", _0)]
    PathAlreadyExists(String),

    #[display("Item {} not found in the trash", _0)]
    ItemNotFound(String),

    #[from]
    XAttr(crate::xattr::XAttrError),
    #[from]
    Io(std::io::Error),
}
