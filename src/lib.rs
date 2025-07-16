use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::loading_screen::loading_screen_plugin;

/// *not* to be confused with [bevy::prelude::LoadState]
pub type LoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod loading_screen;

fn setup_ui(mut commands: Commands, mut scene_builder: SceneBuilder, time: Res<Time>) {
    commands
        .ui_root()
        .spawn_scene(("main", "menu"), &mut scene_builder, |sh| {
            let load_time = time.elapsed_secs();
            sh.get("label")
                .update_text(format!("Menu has loaded ({load_time} seconds)"));

            sh.get("buttons::exit")
                .on_pressed(|mut commands: Commands| {
                    commands.send_event(AppExit::Success);
                });
        });
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins(loading_screen_plugin)
        .load("manifest.cob")
        .add_systems(OnEnter(LoadState::Done), setup_ui);
}
