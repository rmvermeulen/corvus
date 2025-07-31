use std::env;
use std::fmt::Display;
use std::fs::{FileType, read_to_string};
use std::path::PathBuf;
use std::time::Duration;

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::scene_traits::SceneNodeBuilderOuter;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;
use cfg_if::cfg_if;
use derive_more::From;
use itertools::Itertools;
use smol_str::SmolStr;

use crate::loading_screen::loading_screen_plugin;
use crate::view_state::{ViewState, view_state_plugin};

/// *not* to be confused with [bevy::prelude::LoadState]
pub type CobwebLoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod loading_screen;
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

#[derive(Debug, Default, Deref, DerefMut, From, Resource)]
pub struct LocationHistory(Vec<PathBuf>);

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
enum HeaderButton {
    #[default]
    Back,
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
}

#[derive(Clone, Component, Debug, PartialEq)]
enum AppCommand {
    RebuildUi,
    ChangeTab(AppTab),
    // SetPreview(Option<PathBuf>),
    // SetDirectory(PathBuf),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CurrentDirectoryChanged;

#[derive(Clone, Copy, Debug, Default)]
pub struct ReloadCurrentDirectory;

// TODO: implement GoBack
#[derive(Clone, Copy, Debug, Default)]
pub struct GoBack;

// TODO: implement text selection (at least in the address bar)

fn setup_tab_buttons<'a>(
    sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>,
) -> std::result::Result<(), IgnoredError> {
    sh.get("main").on_select(|mut commands: Commands| {
        // TODO: something useful
        commands.change_tab(AppTab::Main);
    });
    sh.get("settings").on_select(|mut commands: Commands| {
        commands.change_tab(AppTab::Settings);
    });
    DONE
}

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

fn setup_header<'a>(header: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(header.path_ends_with(&["header"]));

    let mut reload_button = header.get("location::reload_button");
    #[cfg(feature = "emoji")]
    reload_button.update_text("🔄");
    reload_button.update(|_: TargetId, mut commands: Commands| {
        commands.react().broadcast(ReloadCurrentDirectory);
    });

    let mut back_button = header.get("location::back_button");
    #[cfg(feature = "emoji")]
    back_button.update_text("🔙");
    back_button.update(|_: TargetId, mut commands: Commands| {
        commands.react().broadcast(GoBack);
    });

    let mut location_text = header.get("location::text");
    location_text.update_on(
        broadcast::<CurrentDirectoryChanged>(),
        |id: TargetId, mut text_editor: TextEditor, current_directory: Res<CurrentDirectory>| {
            write_text!(text_editor, *id, "{}", *current_directory);
        },
    );
}

fn init_main_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    setup_header(&mut sh.get("header"));

    for (entry_type, entry) in std::fs::read_dir(".")
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| {
            let ft = entry.file_type().unwrap();
            (EntryType::from(ft), entry)
        })
        .sorted_by_key(|pair| pair.0)
    {
        let path = entry.path();
        let menu_command = match entry_type {
            EntryType::File | EntryType::Symlink => ExplorerCommand::SetPreview(Some(path.clone())),
            EntryType::Directory => ExplorerCommand::SetDirectory(path.clone()),
            _ => unimplemented!("handling unknown entry"),
        };
        sh.get("content::overview::items")
            .spawn_scene(("widgets", "button"), |sh| {
                sh.insert(entry_type).on_pressed(broadcast_fn(menu_command));
                let label = path.to_string_lossy();
                sh.get("text")
                    .update_text(format!("[{}] {label}", entry_type.get_char()));
            });
    }

    sh.get("content::preview").update_on(
        broadcast::<ExplorerCommand>(),
        |id: TargetId, mut commands: Commands, broadcast_event: BroadcastEvent<ExplorerCommand>| {
            if let Ok(ExplorerCommand::SetPreview(path)) = broadcast_event.try_read() {
                commands.entity(*id).despawn_related::<Children>();
                if let Some(path) = path {
                    match read_to_string(path) {
                        Ok(text) => {
                            commands.ui_builder(*id).spawn(Text::new(text));
                        }
                        Err(error) => {
                            commands
                                .ui_builder(*id)
                                .spawn((Text::new(format!("{error}")), TextColor::from(css::RED)));
                        }
                    }
                }
            }
        },
    );
}

fn init_settings_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    let resolution_label = sh.get("settings::resolution::label").id();
    let mut shim = sh.get("settings::resolution::options::view::shim");
    for resolution in &["80x60", "800x600", "1024x768", "1920x1080"] {
        shim.update(
            move |id: TargetId, mut commands: Commands, mut scene_builder: SceneBuilder| {
                commands.ui_builder(*id).spawn_scene(
                    ("widgets", "list_option"),
                    &mut scene_builder,
                    |sh| {
                        // set button text
                        sh.update_text(*resolution);
                        // set value label
                        sh.on_select(move |mut commands: Commands| {
                            commands
                                .get_entity(resolution_label)?
                                .update_text(*resolution);
                            DONE
                        });
                    },
                );
            },
        );
    }
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

fn update_explorer_on_broadcast(
    id: TargetId,
    broadcast_event: BroadcastEvent<ExplorerCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    mut current_directory: ResMut<CurrentDirectory>,
    mut location_history: ResMut<LocationHistory>,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
        ExplorerCommand::Reload => {
            // TODO: re-read_dir current location
        }
        ExplorerCommand::SetDirectory(path) => {
            // TODO: move to a "main-tab-command"?
            let path: PathBuf = if !path.is_absolute() {
                current_directory.join(path)
            } else {
                path.clone()
            }
            .canonicalize()
            .expect("path cannot be canonicalized");
            info!("SetDirectory {path:?}");
            match env::set_current_dir(&path) {
                Ok(_) => {
                    location_history.push(current_directory.clone());
                    **current_directory = path.clone();
                }
                Err(e) => warn!("SetDirectory({path:?}) ERROR: {e:?}"),
            };
        }
        ExplorerCommand::SetPreview(_) => {
            // TODO: move to a "main-tab-command"? anyway, not handled here
        }
    }
}

fn update_tab_content_on_broadcast(
    id: TargetId,
    broadcast_event: BroadcastEvent<AppCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    mut current_directory: ResMut<CurrentDirectory>,
    mut location_history: ResMut<LocationHistory>,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
        AppCommand::RebuildUi => {
            commands.set_state(ViewState::Reset);
        }
        AppCommand::ChangeTab(tab) => {
            let id = *id;
            // clear current tree
            commands.entity(id).despawn_related::<Children>();

            match tab {
                AppTab::Main => {
                    commands.ui_builder(id).spawn_scene(
                        ("main", "main_tab"),
                        &mut scene_builder,
                        init_main_tab,
                    );
                }
                AppTab::Settings => {
                    commands.ui_builder(id).spawn_scene(
                        ("main", "settings_tab"),
                        &mut scene_builder,
                        init_settings_tab,
                    );
                }
            }
        }
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

fn setup_footer<'a>(
    footer: &mut SceneHandle<'a, UiBuilder<'a, Entity>>,
    first_load_time: &mut Option<Duration>,
    time: &Time,
) {
    assert!(footer.path_ends_with(&["footer"]));
    footer.get("text").update_text({
        let load_time = first_load_time.get_or_insert(time.elapsed());
        format!("Loaded in {} seconds", load_time.as_secs_f32())
    });
    footer
        .get("refresh_button")
        .on_pressed(broadcast_fn(AppCommand::RebuildUi));
}

fn setup_ui(
    mut first_load_time: Local<Option<Duration>>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    time: Res<Time>,
) {
    commands
        .ui_root()
        .spawn_scene(("main", "root"), &mut scene_builder, |sh| {
            setup_footer(&mut sh.get("footer"), &mut first_load_time, &time);

            sh.edit("tab_buttons", setup_tab_buttons);

            sh.get("tab_content")
                .update_on(broadcast::<AppCommand>(), update_tab_content_on_broadcast)
                .update_on(broadcast::<ExplorerCommand>(), update_explorer_on_broadcast);

            sh.react().broadcast(AppCommand::ChangeTab(AppTab::Main));

            sh.despawn_on_broadcast::<DespawnUi>();
        });
}

pub fn root_plugin(app: &mut App) {
    app.insert_resource(CurrentDirectory::from(
        env::current_dir().unwrap_or_default(),
    ))
    .init_resource::<LocationHistory>()
    .add_plugins((DefaultPlugins, CobwebUiPlugin))
    .register_component_type::<Marker>()
    .register_component_type::<HeaderButton>()
    .add_plugins((loading_screen_plugin, view_state_plugin))
    .load("manifest.cob")
    .add_systems(
        FixedUpdate,
        (
            broadcast_fn(CurrentDirectoryChanged),
            broadcast_fn(AppCommand::ChangeTab(AppTab::Main)).chain(),
        )
            .run_if(resource_changed::<CurrentDirectory>),
    )
    .add_systems(OnEnter(ViewState::Stable), setup_ui)
    .add_systems(OnEnter(ViewState::Reset), |mut commands: Commands| {
        debug!("despawn ui");
        commands.react().broadcast(DespawnUi);
    });
}
