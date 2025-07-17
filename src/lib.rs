use bevy::prelude::*;
use bevy_cobweb_ui::{prelude::*, sickle::UpdateTextExt};

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
                    DONE
                });

            let res_label = sh.get("settings::resolution::label").id();
            for res in &["800x600", "1024x768", "1920x1080"] {
                let path = format!("settings::resolution::options::view::shim::res_{res}");
                sh.get(path).on_select(move |mut commands: Commands| {
                    commands.get_entity(res_label)?.update_text(*res);
                    DONE
                });
            }

            // Get entity to place in our scene
            let tab_content_entity = sh.get("tab_content").id();

            // set up info tab
            sh.edit("tab_menu::info", |scene_handle| {
                scene_handle.on_select(move |mut c: Commands, mut s: SceneBuilder| {
                    c.get_entity(tab_content_entity)
                        .unwrap()
                        .despawn_related::<Children>();
                    // Use this instead of c.get_entity()
                    c.ui_builder(tab_content_entity)
                        .spawn_scene_simple(("main", "info_tab"), &mut s);
                });
            });

            // set up exit tab
            sh.edit("tab_menu::exit", |scene_handle| {
                scene_handle.on_select(move |mut c: Commands, mut s: SceneBuilder| {
                    c.get_entity(tab_content_entity)
                        .unwrap()
                        .despawn_related::<Children>();
                    // Use this instead of c.get_entity()
                    c.ui_builder(tab_content_entity).spawn_scene(
                        ("main", "exit_tab"),
                        &mut s,
                        |sh| {
                            sh.on_pressed(|mut commands: Commands| {
                                commands.send_event(AppExit::Success);
                            });
                        },
                    );
                });
            });
        });
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins(loading_screen_plugin)
        .load("manifest.cob")
        .add_systems(OnEnter(LoadState::Done), setup_ui);
}
