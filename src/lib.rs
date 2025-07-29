use std::env;
use std::fs::{FileType, read_to_string};
use std::path::PathBuf;
use std::time::Duration;

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;
use cfg_if::cfg_if;
use derive_more::From;
use itertools::Itertools;

use crate::loading_screen::loading_screen_plugin;
use crate::view_state::{ViewState, view_state_plugin};

/// *not* to be confused with [bevy::prelude::LoadState]
pub type CobwebLoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod loading_screen;
pub mod view_state;

trait ChangeTabExt {
    fn change_tab(&mut self, tab: MenuTab);
}

impl<'w, 's> ChangeTabExt for ReactCommands<'w, 's> {
    fn change_tab(&mut self, tab: MenuTab) {
        self.broadcast(MenuCommand::ChangeTab(tab));
    }
}

impl<'w, 's> ChangeTabExt for Commands<'w, 's> {
    fn change_tab(&mut self, tab: MenuTab) {
        self.react().change_tab(tab);
    }
}

#[derive(Debug, Deref, DerefMut, From, Resource)]
pub struct CurrentDirectory(Option<PathBuf>);

impl CurrentDirectory {
    fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self(Some(path.into()))
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

#[derive(Clone, Component, Copy, Debug, PartialEq)]
enum MenuTab {
    Main,
    Settings,
}

#[derive(Clone, Component, Debug, PartialEq)]
enum MenuCommand {
    Refresh,
    SetPreview(Option<PathBuf>),
    ChangeTab(MenuTab),
    SetDirectory(PathBuf),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CurrentDirectoryChanged;

fn setup_tab_buttons<'a>(
    sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>,
) -> std::result::Result<(), IgnoredError> {
    sh.get("main").on_select(|mut commands: Commands| {
        // TODO: something useful
        commands.change_tab(MenuTab::Main);
    });
    sh.get("settings").on_select(|mut commands: Commands| {
        commands.change_tab(MenuTab::Settings);
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

fn init_main_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    sh.get("header::directory::text").update_on(
        broadcast::<CurrentDirectoryChanged>(),
        |id: TargetId, mut text_editor: TextEditor, current_directory: Res<CurrentDirectory>| {
            let text = current_directory
                .clone()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "[invalid directory]".to_string());
            write_text!(text_editor, *id, "{text}");
        },
    );

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
            EntryType::File | EntryType::Symlink => MenuCommand::SetPreview(Some(path.clone())),
            EntryType::Directory => MenuCommand::SetDirectory(path.clone()),
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
        broadcast::<MenuCommand>(),
        |id: TargetId, mut commands: Commands, broadcast_event: BroadcastEvent<MenuCommand>| {
            if let Ok(MenuCommand::SetPreview(path)) = broadcast_event.try_read() {
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
    // "inspect" current nodes
    let mut shim = sh.get("settings::foo_bar::options::view::shim");
    shim.update(
        |id: TargetId,
         mut text_editor: TextEditor,
         markers: Query<&Marker>,
         children: Query<&Children>| {
            for (i, child) in children.iter_descendants(*id).enumerate() {
                match markers.get(child) {
                    Ok(Marker::Option) => {
                        write_text!(text_editor, child, "lmao#{i} from code!");
                    }
                    Ok(marker) => {
                        warn!("unexpected marker: {marker:?}");
                    }
                    Err(error) => {
                        error!("for {child:?} -> {error:?}");
                    }
                }
            }
        },
    );
}

fn update_tab_content_on_broadcast(
    id: TargetId,
    broadcast_event: BroadcastEvent<MenuCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    mut current_directory: ResMut<CurrentDirectory>,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
        MenuCommand::Refresh => {
            commands.set_state(ViewState::Reset);
        }
        MenuCommand::SetDirectory(path) => {
            // TODO: move to a "main-tab-command"?
            let path: PathBuf = if !path.is_absolute()
                && let Some(cwd) = current_directory.take()
            {
                cwd.join(path)
            } else {
                path.clone()
            }
            .canonicalize()
            .expect("path cannot be canonicalized");
            info!("SetDirectory {path:?}");
            match env::set_current_dir(&path) {
                Ok(_) => {
                    *current_directory = CurrentDirectory::from_path(path);
                }
                Err(e) => warn!("SetDirectory({path:?}) ERROR: {e:?}"),
            };
        }
        MenuCommand::SetPreview(_) => {
            // TODO: move to a "main-tab-command"? anyway, not handled here
        }
        MenuCommand::ChangeTab(tab) => {
            let id = *id;
            // clear current tree
            commands.entity(id).despawn_related::<Children>();

            match tab {
                MenuTab::Main => {
                    commands.ui_builder(id).spawn_scene(
                        ("main", "main_tab"),
                        &mut scene_builder,
                        init_main_tab,
                    );
                }
                MenuTab::Settings => {
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

fn setup_ui(
    mut first_load_time: Local<Option<Duration>>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    time: Res<Time>,
) {
    commands
        .ui_root()
        .spawn_scene(("main", "root"), &mut scene_builder, |sh| {
            {
                let load_time = first_load_time.get_or_insert(time.elapsed());
                let load_time_label = format!("Loaded in {} seconds", load_time.as_secs_f32());
                sh.get("title").update_text(load_time_label);
            }

            sh.get("refresh").on_pressed(|mut commands: Commands| {
                commands.react().broadcast(MenuCommand::Refresh);
            });

            sh.edit("tab_buttons", setup_tab_buttons);

            sh.get("tab_content")
                .update_on(broadcast::<MenuCommand>(), update_tab_content_on_broadcast);

            sh.react().broadcast(MenuCommand::ChangeTab(MenuTab::Main));

            sh.despawn_on_broadcast::<DespawnUi>();
        });
}

pub fn root_plugin(app: &mut App) {
    app.insert_resource(CurrentDirectory::from(env::current_dir().ok()))
        .add_plugins((DefaultPlugins, CobwebUiPlugin))
        .register_component_type::<Marker>()
        .add_plugins((loading_screen_plugin, view_state_plugin))
        .load("manifest.cob")
        .add_systems(
            FixedUpdate,
            (
                broadcast_fn(CurrentDirectoryChanged),
                broadcast_fn(MenuCommand::ChangeTab(MenuTab::Main)).chain(),
            )
                .run_if(resource_changed::<CurrentDirectory>),
        )
        .add_systems(OnEnter(ViewState::Stable), setup_ui)
        .add_systems(OnEnter(ViewState::Reset), |mut commands: Commands| {
            debug!("despawn ui");
            commands.react().broadcast(DespawnUi);
        });
}
