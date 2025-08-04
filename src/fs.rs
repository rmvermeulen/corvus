use std::fs::FileType;

use crate::prelude::*;

#[derive(Clone, Component, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
    Unknown,
}

impl EntryType {
    pub(crate) fn get_char(&self) -> char {
        cfg_if! {
            if #[cfg(feature = "emoji")] {
                match self {
                    Self::Directory => '📁',
                    Self::File => '📄',
                    Self::Symlink => '🔗',
                    Self::Unknown => '❓',
                }
            } else {
                match self {
                    Self::Directory => 'd',
                    Self::File => 'f',
                    Self::Symlink => 's',
                    Self::Unknown => 'u',
                }
            }
        }
    }
}

impl From<FileType> for EntryType {
    fn from(ft: FileType) -> Self {
        if ft.is_dir() {
            Self::Directory
        } else if ft.is_file() {
            Self::File
        } else if ft.is_symlink() {
            Self::Symlink
        } else {
            Self::Unknown
        }
    }
}

pub fn fs_plugin(app: &mut App) {
    // get fs info asap, even if ui is still doing things
    app.add_systems(Startup, || {});
}
