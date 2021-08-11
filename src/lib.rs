use serde::{Deserialize, Serialize};
use digest::Digest;
use std::io::Read;

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct FileHashHex {
    hash: String,
}

impl std::convert::TryFrom<FileHashHex> for FileHash {
    type Error = hex::FromHexError;
    fn try_from(data: FileHashHex) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: hex::decode(data.hash)?.into_boxed_slice(),
        })
    }
}

impl std::convert::From<FileHash> for FileHashHex {
    fn from(hash: FileHash) -> FileHashHex {
        FileHashHex {
            hash: hex::encode(hash.hash),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "FileHashHex")]
#[serde(into = "FileHashHex")]
pub struct FileHash {
    hash: Box<[u8]>,
}

impl FileHash {
    pub fn new<D: Digest + std::io::Write, P: AsRef<std::path::Path>>(
        path: P,
        hasher: &mut D,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(&path)?;
        std::io::copy(&mut file, hasher)?;
        Ok(Self {
            hash: hasher
                .finalize_reset()
                .as_slice()
                .to_vec()
                .into_boxed_slice(),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum FsItem {
    File(FileHash),
    Directory(Contents),
}

impl FsItem {
    fn create(path: &std::path::Path, item: FsItem) -> Self {
        let mut new = item;
        let mut components = path.components().peekable();
        while let Some(next) = components.next_back() {
            if components.peek().is_some() {
                let mut new2: Contents = Default::default();
                let next: &std::path::Path = next.as_ref();
                new2.items.insert(next.to_path_buf().into_boxed_path(), new);
                new = FsItem::Directory(new2);
            }
        }
        new
    }
    fn as_file(&self) -> Option<&FileHash> {
        if let FsItem::File(file) = self {
            Some(file)
        } else {
            None
        }
    }
    fn as_file_mut(&mut self) -> Option<&mut FileHash> {
        if let FsItem::File(file) = self {
            Some(file)
        } else {
            None
        }
    }
    fn as_directory(&self) -> Option<&Contents> {
        if let FsItem::Directory(file) = self {
            Some(file)
        } else {
            None
        }
    }
    fn as_directory_mut(&mut self) -> Option<&mut Contents> {
        if let FsItem::Directory(file) = self {
            Some(file)
        } else {
            None
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Contents {
    items: std::collections::HashMap<Box<std::path::Path>, FsItem>,
}

impl Contents {
    fn add_item(&mut self, path: &std::path::Path, item: FsItem) {
        let mut components = path.components();
        if let Some(base) = components.next() {
            let base: &std::path::Path = base.as_ref();
            let without_base = components.as_path();
            if components.next().is_some() {
                if let Some(dir) = self
                    .items
                    .get_mut(base)
                    .and_then(|item| item.as_directory_mut())
                {
                    dir.add_item(without_base, item)
                } else {
                    self.items.insert(
                        base.to_path_buf().into_boxed_path(),
                        FsItem::create(without_base, item),
                    );
                    return;
                };
            } else {
                if item.as_directory().is_some() && self.items.contains_key(base) {
                    return;
                }
                self.items
                    .insert(base.to_path_buf().into_boxed_path(), item);
            }
        }
    }

    pub fn add_file(&mut self, path: &std::path::Path, hash: FileHash) {
        self.add_item(path, FsItem::File(hash));
    }

    pub fn add_dir(&mut self, path: &std::path::Path) {
        self.add_item(path, FsItem::Directory(Default::default()));
    }
}
