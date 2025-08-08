use std::env::{self, current_dir};
use std::fs::{FileType, read_dir, read_link};
use std::io::{self, ErrorKind};

use bevy::tasks::{IoTaskPool, Task, block_on, poll_once};
use smol_str::SmolStr;

use crate::bridge::CurrentDirectoryChanged;
use crate::config::ICON_CONFIG;
use crate::prelude::{Event, *};
use crate::resources::{CurrentDirectory, DirectoryEntries, LocationHistory};
use crate::traits::WithUiIcon;

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

#[derive(Clone, Debug, Eq, PartialEq)]
struct NodeInfo {
    name: String,
    path: PathBuf,
}

impl PartialOrd for NodeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum EntryTypeData {
    Directory,
    File,
    Link(NodeInfo),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedEntry {
    info: NodeInfo,
    entry_type: EntryTypeData,
}

impl ResolvedEntry {
    pub fn path(&self) -> &Path {
        self.info.path.as_path()
    }
    pub fn entry_type_data(&self) -> &EntryTypeData {
        &self.entry_type
    }
    pub fn entry_type(&self) -> EntryType {
        match &self.entry_type {
            EntryTypeData::Directory => EntryType::Directory,
            EntryTypeData::File => EntryType::File,
            EntryTypeData::Link(_) => EntryType::Symlink,
        }
    }
}

impl PartialOrd for ResolvedEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let info = self.info.partial_cmp(&other.info);
        self.entry_type()
            .partial_cmp(&other.entry_type())
            .map(|entry_type| {
                if let Some(info) = info {
                    entry_type.then(info)
                } else {
                    entry_type
                }
            })
            .or(info)
    }
}

impl Ord for ResolvedEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .unwrap_or_else(|| self.info.path.cmp(&other.info.path))
    }
}

#[derive(Clone, Debug, Event)]
pub enum FsEvent {
    DirectoryChanged(PathBuf),
    NotADirectory(PathBuf),
    DirectoryResolved { path: PathBuf, entity: Entity },
    IoError(String),
}

#[derive(Clone, Debug, Event)]
pub enum FsCommand {
    ChangeDirectory(PathBuf),
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
            read_dir(path).map_err(|e| e.to_string()).map(|entries| {
                entries
                    .flatten()
                    .filter_map(resolve_entry)
                    .collect::<Vec<_>>()
            })
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
    mut fs_events: EventWriter<FsEvent>,
    loaders: Query<(Entity, &mut Loader)>,
) {
    for (e, mut loader) in loaders {
        if let Some(result) = block_on(poll_once(&mut loader.task)) {
            let event = match result {
                Ok(entries) => {
                    let path = loader.path.clone();
                    commands
                        .entity(e)
                        .remove::<Loader>()
                        .insert(LoadedDirectory {
                            path: path.clone(),
                            entries,
                        });
                    FsEvent::DirectoryResolved {
                        path: path.clone(),
                        entity: e,
                    }
                }
                Err(message) => {
                    error!("Loader Error: {message}");
                    FsEvent::IoError(message)
                }
            };
            fs_events.write(event);
        }
    }
}
fn update_directory_entries(
    mut events: EventReader<FsEvent>,
    cwd: Res<CurrentDirectory>,
    mut entries: ResMut<DirectoryEntries>,
    loaded_directories: Query<&LoadedDirectory>,
) {
    for event in events.read() {
        if let FsEvent::DirectoryResolved { path, entity } = event.clone()
            && path == **cwd
            && let Ok(directory) = loaded_directories.get(entity)
            && directory.path == path
        {
            **entries = directory.entries.clone();
        }
    }
}

fn handle_fs_commands(
    mut commands: Commands,
    mut fs_commands: EventReader<FsCommand>,
    mut current_directory: ResMut<CurrentDirectory>,
    mut location_history: ResMut<LocationHistory>,
) {
    for command in fs_commands.read() {
        match command {
            FsCommand::ChangeDirectory(path) => {
                let path: PathBuf = if !path.is_absolute() {
                    current_directory.join(path)
                } else {
                    path.to_owned()
                }
                .canonicalize()
                .expect("path cannot be canonicalized");
                if path == **current_directory {
                    return;
                }
                info!("SetDirectory {path:?}");
                match env::set_current_dir(&path) {
                    Ok(_) => {
                        location_history.back.push(current_directory.clone());
                        **current_directory = path.clone();
                        let task = read_dir_task(&path);
                        commands.spawn(Loader { path, task });
                        commands.send_event(CurrentDirectoryChanged);
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::NotADirectory {
                            info!("not a directory, opening as preview");
                            commands.send_event(FsEvent::NotADirectory(path));
                        } else {
                            warn!("SetDirectory({path:?}) ERROR: {e:?}")
                        }
                    }
                };
            }
        }
    }
}

pub fn fs_plugin(app: &mut App) {
    app.init_resource::<DirectoryEntries>()
        .insert_resource(CurrentDirectory::from(
            current_dir().expect("no current working directory?!"),
        ))
        .add_event::<FsEvent>()
        .add_event::<FsCommand>()
        .add_systems(
            Startup,
            // fs does not wait for ui
            startup_fs_plugin,
        )
        .add_systems(
            FixedUpdate,
            (
                poll_loader_tasks,
                update_directory_entries
                    .run_if(on_event::<FsEvent>.or(resource_changed::<CurrentDirectory>)),
                handle_fs_commands.run_if(on_event::<FsCommand>),
            ),
        );
}
