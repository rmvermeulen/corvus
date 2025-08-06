use std::env;
use std::io::ErrorKind;
use std::time::Duration;

use crate::fs::CurrentDirectoryChanged;
use crate::loading_screen::loading_screen_plugin;
use crate::prelude::*;
use crate::resources::{CurrentDirectory, PanelLayout};
use crate::traits::{ChangeTabExt, PathChecksExt};
use crate::ui::ui_events::ViewStateReset;
use crate::ui::view_state::{ViewState, view_state_plugin};
use crate::{LocationHistory, PreviewPath};

mod main_tab;
mod settings_tab;
mod ui_events;
mod view_state;

pub fn send_event_fn<E: Event + Clone + Send + Sync + 'static>(
    value: E,
) -> impl Fn(EventWriter<E>) {
    move |mut writer: EventWriter<E>| {
        writer.write(value.clone());
    }
}

pub fn broadcast_fn<T: Clone + Send + Sync + 'static>(value: T) -> impl Fn(Commands) {
    move |mut commands| {
        commands.react().broadcast(value.clone());
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

#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(CobwebLoadState = CobwebLoadState::Done)]
pub enum AppTab {
    #[default]
    Main,
    Settings,
}

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) enum ExplorerCommand {
    Reload,
    SetPreview(Option<PathBuf>),
    SetDirectory(PathBuf),
    HistoryBack,
    HistoryNext,
    GotoParent,
}

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) enum AppCommand {
    RebuildUi,
    ChangeTab(AppTab),
    // SetPreview(Option<PathBuf>),
    // SetDirectory(PathBuf),
}

// TODO: implement text selection (at least in the address bar)

fn setup_tab_buttons<'a>(
    sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>,
) -> std::result::Result<(), IgnoredError> {
    sh.get("main")
        .on_select(|mut commands: Commands| {
            // TODO: something useful
            commands.change_tab(AppTab::Main);
        })
        .update(
            // select main tab by default
            move |id: TargetId, mut commands: Commands| {
                commands.react().entity_event(*id, Select);
            },
        );
    sh.get("settings").on_select(|mut commands: Commands| {
        commands.change_tab(AppTab::Settings);
    });
    DONE
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

fn update_tab_content_on_app_command(
    id: TargetId,
    broadcast_event: BroadcastEvent<AppCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    mut next_app_tab: ResMut<NextState<AppTab>>,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
        AppCommand::RebuildUi => {
            // drop and rebuild ui
            commands.set_state(ViewState::Unstable);
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
                        main_tab::init_main_tab,
                    );
                }
                AppTab::Settings => {
                    commands.ui_builder(id).spawn_scene(
                        ("tabs_settings", "settings_tab"),
                        &mut scene_builder,
                        settings_tab::init_settings_tab,
                    );
                }
            }

            next_app_tab.set(*tab);
        }
    }
}

pub fn build_ui(
    mut first_load_time: Local<Option<Duration>>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
    time: Res<Time>,
    active_tab: Res<State<AppTab>>,
) {
    commands
        .ui_root()
        .spawn_scene(("main", "root"), &mut scene_builder, |root| {
            setup_footer(&mut root.get("footer"), &mut first_load_time, &time);

            root.edit("tab_buttons", setup_tab_buttons);

            root.get("tab_content")
                .update_on(broadcast::<AppCommand>(), update_tab_content_on_app_command)
                .update_on(
                    broadcast::<ExplorerCommand>(),
                    update_explorer_on_explorer_command,
                );

            let tab = *active_tab.get();
            root.react().broadcast(AppCommand::ChangeTab(tab));

            root.despawn_on_broadcast::<ViewStateReset>();
        });
}

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
            commands.insert_resource(PreviewPath::from(
                preview_path
                    .as_ref()
                    .and_then(|path| path.canonicalize().ok()),
            ));
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

fn clear_preview_path(mut preview_path: ResMut<PreviewPath>) {
    _ = preview_path.take();
}

pub fn ui_plugin(app: &mut App) {
    app.add_plugins(CobwebUiPlugin)
        .load("cobweb/manifest.cob")
        .add_plugins((loading_screen_plugin, view_state_plugin))
        .add_sub_state::<AppTab>()
        .init_resource::<PanelLayout>()
        .init_resource::<LocationHistory>()
        .init_resource::<PreviewPath>()
        .register_component_type::<Marker>()
        .register_component_type::<NavigationButton>()
        .add_systems(
            FixedUpdate,
            (
                (
                    // re-build tab
                    broadcast_fn(AppCommand::ChangeTab(AppTab::Main)),
                    // clear whatever is the preview path
                    clear_preview_path,
                )
                    .run_if(on_event::<CurrentDirectoryChanged>),
                broadcast_fn(ui_events::UpdatePreview).run_if(resource_changed::<PreviewPath>),
            ),
        )
        .add_systems(OnEnter(ViewState::Stable), build_ui)
        .add_systems(OnEnter(ViewState::Unstable), |mut commands: Commands| {
            debug!("despawn ui");
            commands.react().broadcast(ViewStateReset);
        });
}
