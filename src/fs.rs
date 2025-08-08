use std::env::current_dir;
use std::fs::{FileType, read_dir, read_link};
use std::io;

use bevy::tasks::{IoTaskPool, Task, block_on, poll_once};
use smol_str::SmolStr;

use crate::config::ICON_CONFIG;
use crate::prelude::{Event, *};
use crate::resources::CurrentDirectory;
use crate::traits::WithUiIcon;
use crate::ui::send_event_fn;

#[derive(Clone, Component, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
    Unknown,
}

impl WithUiIcon for EntryType {
    fn get_icon(&self) -> SmolStr {
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
struct NodeInfo {
    name: String,
    path: PathBuf,
}

impl<P: Into<PathBuf>> From<P> for NodeInfo {
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
    Link(NodeInfo),
}

#[derive(Clone, Debug)]
pub struct ResolvedEntry {
    info: NodeInfo,
    entry_type: EntryTypeData,
}

#[derive(Clone, Copy, Debug, Default, Event)]
pub struct CurrentDirectoryChanged;

#[derive(Clone, Debug, Event)]
pub enum FsEvents {
    DirectoryResolved(PathBuf),
    IoError(String),
}

fn resolve_entry(entry: std::fs::DirEntry) -> Option<ResolvedEntry> {
    let path = entry.path();
    let entry_type = if path.is_file() {
        EntryTypeData::File
    } else if path.is_dir() {
        EntryTypeData::Directory
    } else {
        let link = read_link(path).ok()?;
        EntryTypeData::Link(link.into())
    };
    let data = NodeInfo {
        name: entry.file_name().to_string_lossy().to_string(),
        path: entry.path(),
    };
    Some(ResolvedEntry {
        info: data,
        entry_type,
    })
}

fn read_dir_task<P: Into<PathBuf>>(path: P) -> Task<Result<Vec<ResolvedEntry>, String>> {
    IoTaskPool::get().spawn({
        let path: PathBuf = path.into();
        async move {
            let entries = read_dir(path).map_err(|e| e.to_string())?;
            Ok(entries
                .flatten()
                .filter_map(resolve_entry)
                .collect::<Vec<_>>())
        }
    })
}

fn startup_fs_plugin(mut commands: Commands, cwd: Res<CurrentDirectory>) {
    let path: PathBuf = cwd.clone();
    let task = read_dir_task(&path);
    commands.spawn(Loader { path, task });
}

fn log_error_fn(In(result): In<io::Result<()>>) {
    if let Err(error) = result {
        error!("IO error: {error:?}");
    }
}

fn poll_loader_tasks(
    mut commands: Commands,
    mut fs_events: EventWriter<FsEvents>,
    loaders: Query<(Entity, &mut Loader)>,
) {
    for (e, mut loader) in loaders {
        if let Some(result) = block_on(poll_once(&mut loader.task)) {
            match result {
                Ok(entries) => {
                    let path = loader.path.clone();
                    fs_events.write(FsEvents::DirectoryResolved(path.clone()));
                    commands
                        .entity(e)
                        .remove::<Loader>()
                        .insert(LoadedDirectory { path, entries });
                }
                Err(message) => {
                    error!("Loader Error: {message}");
                    fs_events.write(FsEvents::IoError(message));
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
    .add_event::<FsEvents>()
    .add_systems(
        Startup,
        // fs does not wait for ui
        startup_fs_plugin,
    )
    .add_systems(
        Update,
        (
            poll_loader_tasks,
            send_event_fn(CurrentDirectoryChanged).run_if(resource_changed::<CurrentDirectory>),
        ),
    );
}
