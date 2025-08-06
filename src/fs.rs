use std::env::current_dir;
use std::fs::{FileType, read_dir, read_link};
use std::io;

use bevy::tasks::{IoTaskPool, Task, block_on, poll_once};

use crate::prelude::{Event, *};
use crate::resources::CurrentDirectory;
use crate::ui::send_event_fn;

#[derive(Clone, Copy, Debug)]
pub struct NavigationIconConfig {
    pub back: char,
    pub next: char,
    pub up: char,
    pub reload: char,
}

#[derive(Clone, Copy, Debug)]
pub struct FsIconConfig {
    pub file: char,
    pub directory: char,
    pub symlink: char,
    pub unknown: char,
}

#[derive(Clone, Copy, Debug)]
pub struct IconConfig {
    pub navigation: NavigationIconConfig,
    pub fs: FsIconConfig,
}

cfg_if! {
    if #[cfg(feature = "emoji")] {
        const ICON_CONFIG: IconConfig = IconConfig {
            navigation: NavigationIconConfig {
                back: '🔙',
                next: '🔜',
                up: '🔝',
                reload: '🔄',
            },
            fs: FsIconConfig {
                file: '📄',
                directory: '📁',
                symlink: '🔗',
                unknown: '❓',
            },
        };
    } else {
        const ICON_CONFIG: IconConfig = IconConfig {
            navigation: NavigationIconConfig {
                back: 'b',
                next: 'n',
                up: 'u',
                reload: 'r',
            },
            fs: FsIconConfig {
                file: 'f',
                directory: 'd',
                symlink: 's',
                unknown: 'u',
            },
        };
    }
}

#[derive(Clone, Component, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
    Unknown,
}

impl EntryType {
    pub(crate) fn get_icon(&self) -> char {
        match self {
            Self::Directory => ICON_CONFIG.fs.directory,
            Self::File => ICON_CONFIG.fs.file,
            Self::Symlink => ICON_CONFIG.fs.symlink,
            Self::Unknown => ICON_CONFIG.fs.unknown,
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

#[derive(Clone, Component, Copy, Debug)]
enum IoState {
    Reading,
    Processing,
    Done,
}

#[derive(Clone, Component, Debug)]
struct LoadedDirectory {
    path: PathBuf,
    entries: Vec<ResolvedEntry>,
}

#[derive(Component, Debug)]
struct Loader {
    path: PathBuf,
    task: Task<Result<Vec<ResolvedEntry>, String>>,
}

#[derive(Clone, Copy)]
enum ConcreteNode {
    File,
    Directory,
}

#[derive(Clone, Debug)]
struct NodeData {
    name: String,
    path: PathBuf,
}

impl<P: Into<PathBuf>> From<P> for NodeData {
    fn from(value: P) -> Self {
        let path = value.into();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        Self { name, path }
    }
}

#[derive(Clone, Debug)]
enum EntryTypeData {
    Directory,
    File,
    Link(NodeData),
}

#[derive(Clone, Debug)]
struct ResolvedEntry {
    data: NodeData,
    entry_type: EntryTypeData,
}

#[derive(Clone, Copy, Debug, Default, Event)]
pub struct CurrentDirectoryChanged;

fn startup_fs_plugin(mut commands: Commands) -> std::result::Result<(), io::Error> {
    let path = current_dir()?;
    let task = IoTaskPool::get().spawn({
        let path = path.clone();
        async {
            let entries = read_dir(path).map_err(|e| e.to_string())?;
            Ok(entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let entry_type = if path.is_file() {
                        EntryTypeData::File
                    } else if path.is_dir() {
                        EntryTypeData::Directory
                    } else {
                        let link = read_link(path).ok()?;
                        EntryTypeData::Link(link.into())
                    };
                    let data = NodeData {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path(),
                    };
                    Some(ResolvedEntry { data, entry_type })
                })
                .collect::<Vec<_>>())
        }
    });
    commands.spawn(Loader { path, task });
    Ok(())
}

fn log_error_fn(In(result): In<io::Result<()>>) {
    if let Err(error) = result {
        error!("IO error: {error:?}");
    }
}

fn poll_loader_tasks(
    time: Res<Time>,
    mut commands: Commands,
    loaders: Query<(Entity, &mut Loader)>,
) {
    for (e, mut loader) in loaders {
        if let Some(result) = block_on(poll_once(&mut loader.task)) {
            match result {
                Ok(entries) => {
                    info!("read_dir elapsed: {} seconds", time.elapsed_secs());
                    info!("{entries:?}");
                    commands
                        .entity(e)
                        .remove::<Loader>()
                        .insert(LoadedDirectory {
                            path: loader.path.clone(),
                            entries,
                        });
                }
                Err(message) => {
                    error!("Loader Error: {message}");
                }
            }
        }
    }
}

pub fn fs_plugin(app: &mut App) {
    // TODO: interact with ui_plugin
    app.insert_resource(CurrentDirectory::from(
        current_dir().expect("no current working directory?!"),
    ))
    .add_event::<CurrentDirectoryChanged>()
    .add_systems(
        Startup,
        // fs does not wait for ui
        startup_fs_plugin.pipe(log_error_fn),
    )
    .add_systems(
        Update,
        (
            poll_loader_tasks,
            send_event_fn(CurrentDirectoryChanged).run_if(resource_changed::<CurrentDirectory>),
        ),
    );
}
