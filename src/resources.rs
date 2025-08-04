use std::fmt::Display;
use std::path::PathBuf;

use bevy::prelude::*;
use derive_more::{Display, From};

#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Resource)]
pub enum PanelLayout {
    #[default]
    Automatic,
    Horizontal,
    Vertical,
}

#[derive(Debug, Deref, DerefMut, Resource)]
pub struct CurrentDirectory(PathBuf);

#[derive(Debug, Default, Deref, DerefMut, From, Resource)]
pub struct PreviewPath(Option<PathBuf>);

#[derive(Debug, Default, From, Resource)]
pub struct LocationHistory {
    pub back: Vec<PathBuf>,
    pub next: Vec<PathBuf>,
}

impl From<PathBuf> for CurrentDirectory {
    fn from(mut path: PathBuf) -> Self {
        while let Err(_) = path.canonicalize()
            && path.pop()
        {}
        Self(path)
    }
}

impl Display for CurrentDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self.to_string_lossy();
        write!(f, "{text}")
    }
}
