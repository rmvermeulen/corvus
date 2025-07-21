use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;

use crate::loading_screen::loading_screen_plugin;

/// *not* to be confused with [bevy::prelude::LoadState]
pub type LoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod loading_screen;

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

enum MenuTab {
    Main,
    Settings,
    Exit,
}

enum MenuCommand {
    ChangeTab(MenuTab),
}

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
    sh.get("exit").on_select(|mut commands: Commands| {
        commands.change_tab(MenuTab::Exit);
    });
    DONE
}

fn init_main_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    let mut sh = sh.get("buttons");
    sh.get("start").on_pressed(|mut commands: Commands| {
        // TODO: something useful
        commands.change_tab(MenuTab::Main);
    });
    sh.get("settings").on_pressed(|mut commands: Commands| {
        commands.change_tab(MenuTab::Settings);
    });
    sh.get("exit").on_pressed(|mut commands: Commands| {
        commands.change_tab(MenuTab::Exit);
    });
}

fn init_settings_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    let resolution_label = sh.get("settings::resolution::label").id();
    let mut shim = sh.get("settings::resolution::options::view::shim");
    let id = shim.id();
    for resolution in &["800x600", "1024x768", "1920x1080"] {
        shim.update(
            move |_: TargetId, mut commands: Commands, mut scene_builder: SceneBuilder| {
                commands.ui_builder(id).spawn_scene(
                    ("main", "tab_button"),
                    &mut scene_builder,
                    |sh| {
                        // set button text
                        sh.get("text").update_text(*resolution);
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

fn init_exit_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    sh.get("buttons::exit")
        .on_pressed(|mut commands: Commands| {
            commands.send_event(AppExit::Success);
            DONE
        });
    sh.get("buttons::back")
        .on_pressed(|mut commands: Commands| {
            commands
                .react()
                .broadcast(MenuCommand::ChangeTab(MenuTab::Main));
            DONE
        });
}

fn update_tab_content_on_broadcast(
    id: TargetId,
    broadcast_event: BroadcastEvent<MenuCommand>,
    mut commands: Commands,
    mut scene_builder: SceneBuilder,
) {
    let Ok(event) = broadcast_event.try_read() else {
        return;
    };
    match event {
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
                MenuTab::Exit => {
                    commands.ui_builder(id).spawn_scene(
                        ("main", "exit_tab"),
                        &mut scene_builder,
                        init_exit_tab,
                    );
                }
            }
        }
    }
}

fn setup_ui(mut commands: Commands, mut scene_builder: SceneBuilder, time: Res<Time>) {
    commands
        .ui_root()
        .spawn_scene(("main", "menu"), &mut scene_builder, |sh| {
            let load_time = time.elapsed_secs();
            sh.get("label").update_text(format!("{load_time} seconds"));

            // setup on_pressed for all tabs
            sh.edit("tab_buttons", setup_tab_buttons);

            sh.get("tab_content")
                .update_on(broadcast::<MenuCommand>(), update_tab_content_on_broadcast);

            sh.react().broadcast(MenuCommand::ChangeTab(MenuTab::Main));
        });
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins(loading_screen_plugin)
        .load("manifest.cob")
        .add_systems(OnEnter(LoadState::Done), setup_ui);
}
