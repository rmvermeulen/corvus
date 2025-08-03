use std::env;
use std::fmt::Display;
use std::fs::{FileType, read_to_string};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css;
use bevy::image::{CompressedImageFormats, ImageType};
use bevy::prelude::*;
use bevy::tasks::futures_lite::io;
use bevy::ui::RelativeCursorPosition;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::scene_traits::SceneNodeBuilderOuter;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;
use cfg_if::cfg_if;
use derive_more::{Display, From};
use itertools::Itertools;

use crate::loading_screen::loading_screen_plugin;
use crate::ui_events::{CurrentDirectoryChanged, LocationSelectionUpdated, PreviewPathChanged};
use crate::view_state::{ViewState, view_state_plugin};

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

pub mod ui_events {
    use bevy::utils::default;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct PreviewPathChanged;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct CurrentDirectoryChanged;

    #[derive(Clone, Debug, Default)]
    pub struct LocationSelectionUpdated {
        pub before: String,
        pub selected: String,
        pub after: String,
    }

    impl LocationSelectionUpdated {
        pub fn new_no_selection(text: String) -> Self {
            Self {
                before: text,
                selected: default(),
                after: default(),
            }
        }
        pub fn new(before: String, selected: String, after: String) -> Self {
            Self {
                before,
                selected,
                after,
            }
        }
    }
}

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

fn setup_location_text<'a>(location: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(location.path_ends_with(&["location"]));

    fn split_string(input: &str, index: usize) -> (String, String, String) {
        let mut chars = input.chars();
        let before: String = chars.by_ref().take(index).collect();
        let selected: String = chars.by_ref().take(1).collect();
        let after: String = chars.by_ref().collect();
        assert_eq!(before.len() + selected.len() + after.len(), input.len());
        (before, selected, after)
    }

    location
        .insert(RelativeCursorPosition::default())
        .observe(
            |trigger: Trigger<Pointer<DragStart>>,
             relcurpos: Query<&RelativeCursorPosition, With<Node>>,
             current_directory: Res<CurrentDirectory>,
             mut commands: Commands| {
                debug!("trigger DragStart");
                if let Ok(rcp) = relcurpos.get(trigger.target())
                    && let Some(Vec2 { x, .. }) = rcp.normalized
                {
                    let cwd = current_directory.to_string();
                    let index = (cwd.len() as f32 * x).floor() as usize;
                    let (before, selected, after) = split_string(&cwd, index);
                    commands
                        .react()
                        .broadcast(LocationSelectionUpdated::new(before, selected, after));
                }
            },
        )
        .observe(
            |trigger: Trigger<Pointer<Drag>>,
             relcurpos: Query<&RelativeCursorPosition, With<Node>>,
             current_directory: Res<CurrentDirectory>,
             mut commands: Commands| {
                trace!("trigger Drag");
                if let Ok(rcp) = relcurpos.get(trigger.target())
                    && let Some(Vec2 { x, .. }) = rcp.normalized
                {
                    let cwd = current_directory.to_string();
                    let index = (cwd.len() as f32 * x).floor() as usize;
                    let (before, selected, after) = split_string(&cwd, index);
                    commands
                        .react()
                        .broadcast(LocationSelectionUpdated::new(before, selected, after));
                }
            },
        )
        .observe(
            |trigger: Trigger<Pointer<DragEnd>>,
             relcurpos: Query<&RelativeCursorPosition, With<Node>>,
             current_directory: Res<CurrentDirectory>,
             mut commands: Commands| {
                info!("trigger DragEnd");
                if let Ok(rcp) = relcurpos.get(trigger.target())
                    && let Some(Vec2 { x, .. }) = rcp.normalized
                {
                    let cwd = current_directory.to_string();
                    let index = (cwd.len() as f32 * x).floor() as usize;
                    let (before, selected, after) = split_string(&cwd, index);
                    commands
                        .react()
                        .broadcast(LocationSelectionUpdated::new(before, selected, after));
                }
            },
        )
        .update_on(
            broadcast::<ui_events::CurrentDirectoryChanged>(),
            |_: TargetId, mut commands: Commands, current_directory: Res<CurrentDirectory>| {
                let cwd = current_directory.to_string();
                commands
                    .react()
                    .broadcast(LocationSelectionUpdated::new_no_selection(cwd));
            },
        );
    location.get("before").update_on(
        broadcast::<LocationSelectionUpdated>(),
        |id: TargetId,
         bce: BroadcastEvent<LocationSelectionUpdated>,
         mut text_editor: TextEditor| {
            if let Ok(ev) = bce.try_read() {
                write_text!(text_editor, *id, "{}", ev.before);
            }
        },
    );
    location.get("selected").update_on(
        broadcast::<LocationSelectionUpdated>(),
        |id: TargetId,
         bce: BroadcastEvent<LocationSelectionUpdated>,
         mut text_editor: TextEditor| {
            if let Ok(ev) = bce.try_read() {
                write_text!(text_editor, *id, "{}", ev.selected);
            }
        },
    );
    location.get("after").update_on(
        broadcast::<LocationSelectionUpdated>(),
        |id: TargetId,
         bce: BroadcastEvent<LocationSelectionUpdated>,
         mut text_editor: TextEditor| {
            if let Ok(ev) = bce.try_read() {
                write_text!(text_editor, *id, "{}", ev.after);
            }
        },
    );
}

fn setup_navigation<'a>(navigation: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(navigation.path_ends_with(&["navigation"]));

    let configs = [
        ("back_button", ExplorerCommand::HistoryBack),
        ("next_button", ExplorerCommand::HistoryNext),
        ("up_button", ExplorerCommand::GotoParent),
        ("reload_button", ExplorerCommand::Reload),
    ];

    #[cfg(feature = "emoji")]
    let configs = configs
        .into_iter()
        .zip([
            "🔙", // back
            "🔜", // next
            "🔜", // up
            "🔄", // reload
        ])
        .map(|((a, b), c)| (a, b, c));

    for config in configs {
        let mut button = navigation.get(config.0);
        button.on_pressed(move |mut commands: Commands| {
            let command = config.1.clone();
            info!("{command:?}");
            commands.react().broadcast(command);
        });
        #[cfg(feature = "emoji")]
        button.update_text(config.2);
    }
    setup_location_text(&mut navigation.get("location"));
}

fn setup_header<'a>(header: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(header.path_ends_with(&["header"]));
    setup_navigation(&mut header.get("navigation"));
}

fn init_main_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    info!("init_main_tab ({:?})", env::current_dir());
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
            EntryType::File => Some(ExplorerCommand::SetPreview(Some(path.clone()))),
            EntryType::Directory | EntryType::Symlink => {
                Some(ExplorerCommand::SetDirectory(path.clone()))
            }
            _ => None,
        };
        sh.get("content::overview::items")
            .spawn_scene(("widgets", "button"), |sh| {
                sh.insert(entry_type);
                if let Some(menu_command) = menu_command {
                    sh.on_pressed(broadcast_fn(menu_command));
                }
                let label = path.to_string_lossy();
                sh.get("text")
                    .update_text(format!("[{}] {label}", entry_type.get_char()));
            });
    }

    #[derive(Clone, Copy, Debug)]
    enum PreviewMode<'a> {
        Text,
        Image(&'a str),
        // TODO: Binary,
    }

    sh.get("content::preview").update_on(
        broadcast::<PreviewPathChanged>(),
        |id: TargetId,
         mut commands: Commands,
         preview_path: Res<PreviewPath>,
         mut images: ResMut<Assets<Image>>| {
            commands.entity(*id).despawn_related::<Children>();
            info!("{preview_path:?}");
            if let Some(path) = (*preview_path).as_ref() {
                let preview_mode = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| {
                        matches!(ext, "png" | "jpg" | "webp").then_some(PreviewMode::Image(ext))
                    })
                    .unwrap_or(PreviewMode::Text);

                #[derive(Debug, thiserror::Error)]
                enum ImageError {
                    #[error("io::Error: {0}")]
                    Io(#[from] io::Error),
                    #[error("TextureError: {0}")]
                    Texture(#[from] TextureError),
                }

                fn read_as_text<'a, P: AsRef<Path>>(
                    path: P,
                    mut builder: UiBuilder<'a, Entity>,
                ) -> io::Result<()> {
                    read_to_string(path).map(move |text| {
                        builder.spawn(Text::new(text));
                    })
                }

                fn read_as_image<'a, P: AsRef<Path>>(
                    path: P,
                    ext: &str,
                    images: &mut Assets<Image>,
                    mut builder: UiBuilder<'a, Entity>,
                ) -> Result<(), ImageError> {
                    std::fs::read(path)
                        .map_err(ImageError::from)
                        .and_then(|bytes| {
                            Image::from_buffer(
                                &bytes,
                                ImageType::Extension(ext),
                                CompressedImageFormats::default(),
                                false,
                                bevy::image::ImageSampler::Default,
                                RenderAssetUsages::RENDER_WORLD,
                            )
                            .map_err(ImageError::from)
                        })
                        .map(move |image| {
                            let image = images.add(image);
                            builder.spawn(ImageNode::new(image));
                        })
                }

                fn read_as_binary<'a, P: AsRef<Path>>(
                    _path: P,
                    mut _builder: UiBuilder<'a, Entity>,
                ) -> Result<(), ImageError> {
                    todo!("read_as_binary using xxd")
                }

                match preview_mode {
                    PreviewMode::Text => {
                        read_as_text(path, commands.ui_builder(*id)).map_err(Into::into)
                    }
                    PreviewMode::Image(ext) => {
                        read_as_image(path, ext, &mut images, commands.ui_builder(*id))
                    }
                }
                // TODO: .or_else(|_| read_as_binary(path, commands.ui_builder(*id)))
                .unwrap_or_else(move |error| {
                    commands
                        .ui_builder(*id)
                        .spawn((Text::new(format!("{error}")), TextColor::from(css::RED)));
                });
            }
        },
    );
}

#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Resource)]
enum PanelLayout {
    #[default]
    Automatic,
    Horizontal,
    Vertical,
}

fn init_settings_tab<'a>(settings_tab: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    let resolution_value = settings_tab.get("settings::resolution::header::value").id();
    let mut shim = settings_tab.get("settings::resolution::options::view::shim");
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
                                .get_entity(resolution_value)?
                                .update_text(*resolution);
                            DONE
                        });
                    },
                );
            },
        );
    }

    settings_tab.edit("settings::layout", |layout_settings| {
        let layout_value_label_id = layout_settings.get("header::value").id();
        for layout in [
            PanelLayout::Automatic,
            PanelLayout::Horizontal,
            PanelLayout::Vertical,
        ] {
            let name = layout.to_string();
            let key = name.to_lowercase();
            layout_settings
                .get(format!("options::{key}"))
                .on_select(
                    move |mut commands: Commands, mut panel_layout: ResMut<PanelLayout>| {
                        // update resource
                        *panel_layout = layout;
                        // update label
                        commands
                            .ui_builder(layout_value_label_id)
                            .update_text(&name);
                    },
                )
                // select option that matches Res<PanelLayout>
                .update(
                    move |id: TargetId, mut commands: Commands, panel_layout: Res<PanelLayout>| {
                        if layout == *panel_layout {
                            commands.react().entity_event(*id, Select);
                        }
                    },
                );
        }
    });
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

fn update_tab_content_on_app_command(
    id: TargetId,
    broadcast_event: BroadcastEvent<AppCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    active_tab: Option<Res<ActiveTab>>,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
        AppCommand::RebuildUi => {
            let tab = active_tab.map(|res| res.tab()).unwrap_or(AppTab::Main);
            commands.insert_resource(ActiveTab(tab));
            commands.set_state(ViewState::Reset);
        }
        AppCommand::ChangeTab(tab) => {
            let id = *id;
            // clear current tree
            commands.entity(id).despawn_related::<Children>();

            match tab {
                AppTab::Main => {
                    commands.ui_builder(id).spawn_scene(
                        ("tabs_main", "main_tab"),
                        &mut scene_builder,
                        init_main_tab,
                    );
                    // show the current directory
                    commands.react().broadcast(CurrentDirectoryChanged);
                }
                AppTab::Settings => {
                    commands.ui_builder(id).spawn_scene(
                        ("tabs_main", "settings_tab"),
                        &mut scene_builder,
                        init_settings_tab,
                    );
                }
            }
            commands.insert_resource(ActiveTab(*tab));
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
    active_tab: Option<Res<ActiveTab>>,
) {
    commands
        .ui_root()
        .spawn_scene(("main", "root"), &mut scene_builder, |sh| {
            setup_footer(&mut sh.get("footer"), &mut first_load_time, &time);

            sh.edit("tab_buttons", setup_tab_buttons);

            sh.get("tab_content")
                .update_on(broadcast::<AppCommand>(), update_tab_content_on_app_command)
                .update_on(
                    broadcast::<ExplorerCommand>(),
                    update_explorer_on_explorer_command,
                );

            let tab = active_tab.map(|res| res.tab()).unwrap_or(AppTab::Main);
            sh.react().broadcast(AppCommand::ChangeTab(tab));

            sh.despawn_on_broadcast::<DespawnUi>();
        });
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins((loading_screen_plugin, view_state_plugin))
        .load("manifest.cob")
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
        .add_systems(OnEnter(ViewState::Stable), setup_ui)
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
