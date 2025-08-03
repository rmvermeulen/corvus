use std::env;
use std::fmt::Display;
use std::fs::FileType;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::scene_traits::SceneNodeBuilderOuter;
use bevy_cobweb_ui::prelude::*;
use cfg_if::cfg_if;
use derive_more::{Display, From};
use itertools::Itertools;

use crate::app_ui::{build_ui, update_tab_content_on_app_command};
use crate::loading_screen::loading_screen_plugin;
use crate::view_state::{ViewState, view_state_plugin};

pub type CobwebLoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod app_ui;
pub mod loading_screen;
pub mod main_tab;
pub mod settings_tab;
pub mod ui_events;
pub mod view_state;

trait ChangeTabExt {
    fn change_tab(&mut self, tab: AppTab);
}

impl<'w, 's> ChangeTabExt for ReactCommands<'w, 's> {
    fn change_tab(&mut self, tab: AppTab) {
        self.broadcast(AppCommand::ChangeTab(tab));
    }
}

impl<'w, 's> ChangeTabExt for Commands<'w, 's> {
    fn change_tab(&mut self, tab: AppTab) {
        self.react().change_tab(tab);
    }
}

#[derive(Debug, Deref, DerefMut, Resource)]
pub struct CurrentDirectory(PathBuf);

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct PreviewPath(Option<PathBuf>);

#[derive(Debug, Default, From, Resource)]
pub struct LocationHistory {
    back: Vec<PathBuf>,
    next: Vec<PathBuf>,
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

#[derive(Clone, Component, Copy, Debug, Default, PartialEq, Reflect)]
enum Marker {
    #[default]
    Widget,
    Label,
    Option,
    Input,
    Button,
}

#[derive(Clone, Component, Copy, Debug, Default, PartialEq, Reflect)]
#[require(Marker::Button)]
enum NavigationButton {
    #[default]
    Back,
    Next,
    Up,
    Reload,
}

#[derive(Clone, Component, Copy, Debug, PartialEq)]
enum AppTab {
    Main,
    Settings,
}

#[derive(Clone, Component, Debug, PartialEq)]
enum ExplorerCommand {
    Reload,
    SetPreview(Option<PathBuf>),
    SetDirectory(PathBuf),
    HistoryBack,
    HistoryNext,
    GotoParent,
}

#[derive(Clone, Component, Debug, PartialEq)]
enum AppCommand {
    RebuildUi,
    ChangeTab(AppTab),
    // SetPreview(Option<PathBuf>),
    // SetDirectory(PathBuf),
}

// TODO: implement text selection (at least in the address bar)

#[derive(Clone, Component, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
    Unknown,
}

impl EntryType {
    fn get_char(&self) -> char {
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

pub fn broadcast_fn<T: Clone + Send + Sync + 'static>(value: T) -> impl Fn(Commands) {
    move |mut commands| {
        commands.react().broadcast(value.clone());
    }
}

#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Resource)]
pub enum PanelLayout {
    #[default]
    Automatic,
    Horizontal,
    Vertical,
}

/// example: inspect current nodes
/// ```rust no_run
/// fn inspect_handle<'a>(handle: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
///     fn on_inspect(id: TargetId, mut text_editor: TextEditor, children: Query<&Children>) {
///         for (i, child) in children.iter_descendants(*id).enumerate() {
///             // look up components etc
///         }
///     }
///     assert_eq!(sh.path(), "scroll_list::options::view::shim");
///     handle.update(on_inspect);
/// }
///```

fn update_explorer_on_explorer_command(
    _: TargetId,
    broadcast_event: BroadcastEvent<ExplorerCommand>,
    mut current_directory: ResMut<CurrentDirectory>,
    mut location_history: ResMut<LocationHistory>,
    mut commands: Commands,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    info!("{event:?}");

    fn set_directory(
        path: &Path,
        current_directory: &mut CurrentDirectory,
        location_history: Option<&mut LocationHistory>,
        commands: &mut Commands,
    ) {
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
                if let Some(history) = location_history {
                    history.back.push(current_directory.clone());
                }
                **current_directory = path.clone();
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotADirectory {
                    info!("not a directory, opening as preview");
                    commands
                        .react()
                        .broadcast(ExplorerCommand::SetPreview(Some(path)));
                } else {
                    warn!("SetDirectory({path:?}) ERROR: {e:?}")
                }
            }
        };
    }

    match event {
        ExplorerCommand::Reload => {
            commands.change_tab(AppTab::Main);
        }
        ExplorerCommand::SetDirectory(path) => {
            set_directory(
                path,
                &mut current_directory,
                Some(&mut location_history),
                &mut commands,
            );
        }
        ExplorerCommand::SetPreview(preview_path) => {
            commands.insert_resource(PreviewPath(preview_path.clone()));
            if let Some(preview_dir_path) = preview_path
                .as_ref()
                .and_then(|p| p.parent().map(PathBuf::from))
            {
                set_directory(
                    &preview_dir_path,
                    &mut current_directory,
                    Some(&mut location_history),
                    &mut commands,
                );
            }
        }
        ExplorerCommand::HistoryBack => {
            if let Some(prev) = location_history.back.pop() {
                location_history.next.insert(0, current_directory.clone());
                set_directory(&prev, &mut current_directory, None, &mut commands);
            }
        }
        ExplorerCommand::HistoryNext => {
            if let Some(next) = location_history.next.pop() {
                location_history.back.push(current_directory.clone());
                set_directory(&next, &mut current_directory, None, &mut commands);
            }
        }
        ExplorerCommand::GotoParent => {
            if let Some(parent) = current_directory.parent().map(Path::to_owned) {
                set_directory(
                    &parent,
                    &mut current_directory,
                    Some(&mut location_history),
                    &mut commands,
                );
            };
        }
    }
}

#[derive(Clone, Copy, Debug, Deref, DerefMut, Resource)]
struct ActiveTab(AppTab);

impl ActiveTab {
    fn tab(&self) -> AppTab {
        **self
    }
}

struct DespawnUi;

pub trait PathChecksExt {
    fn path_vec(&self) -> Vec<&str>;
    fn path_ends_with(&self, needle: &[&str]) -> bool {
        self.path_vec().ends_with(needle)
    }
}

impl<'a, B: SceneNodeBuilderOuter<'a>> PathChecksExt for SceneHandle<'a, B> {
    fn path_vec(&self) -> Vec<&str> {
        self.path().path.iter().collect_vec()
    }
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins((loading_screen_plugin, view_state_plugin))
        .load("cobweb/manifest.cob")
        .add_systems(
            FixedUpdate,
            (
                (
                    broadcast_fn(ui_events::CurrentDirectoryChanged),
                    broadcast_fn(AppCommand::ChangeTab(AppTab::Main)),
                )
                    .chain()
                    .run_if(resource_changed::<CurrentDirectory>),
                broadcast_fn(ui_events::PreviewPathChanged).run_if(resource_changed::<PreviewPath>),
            ),
        )
        .add_systems(OnEnter(ViewState::Stable), build_ui)
        .add_systems(OnEnter(ViewState::Reset), |mut commands: Commands| {
            debug!("despawn ui");
            commands.react().broadcast(DespawnUi);
        })
        .insert_resource(CurrentDirectory::from(
            env::current_dir().unwrap_or_default(),
        ))
        .init_resource::<PanelLayout>()
        .init_resource::<LocationHistory>()
        .init_resource::<PreviewPath>()
        .register_component_type::<Marker>()
        .register_component_type::<NavigationButton>();
}
