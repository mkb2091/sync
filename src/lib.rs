use serde::{Deserialize, Serialize};
use sha2::Digest;
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
    pub fn new<D: Digest, P: AsRef<std::path::Path>>(
        path: P,
        buffer: &mut [u8],
        hasher: &mut D,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(&path)?;
        loop {
            let n = file.read(buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..std::cmp::max(n, buffer.len())]);
        }
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
                if let Some(FsItem::Directory(existing)) = self.items.get_mut(base) {
                    existing.add_item(without_base, item);
                } else {
                    let mut new: Self = Default::default();
                    new.add_item(without_base, item);
                    self.items
                        .insert(base.to_path_buf().into_boxed_path(), FsItem::Directory(new));
                }
            } else {
                if let FsItem::Directory(_) = item {
                    if self.items.contains_key(base) {
                        return;
                    }
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
