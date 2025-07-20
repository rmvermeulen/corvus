use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;

use crate::loading_screen::loading_screen_plugin;

/// *not* to be confused with [bevy::prelude::LoadState]
pub type LoadState = bevy_cobweb_ui::prelude::LoadState;

pub mod loading_screen;

fn setup_ui(mut commands: Commands, mut scene_builder: SceneBuilder, time: Res<Time>) {
    commands
        .ui_root()
        .spawn_scene(("main", "menu"), &mut scene_builder, |sh| {
            let load_time = time.elapsed_secs();
            sh.get("label").update_text(format!("{load_time} seconds"));

            sh.get("buttons::exit")
                .on_pressed(|mut commands: Commands| {
                    commands.send_event(AppExit::Success);
                    DONE
                });

            // Get entity to place in our scene
            let tab_content_entity = sh.get("tab_content").id();

            // set up info tab
            sh.edit("tab_menu::main", |scene_handle| {
                scene_handle.on_select(
                    move |mut c: Commands, mut s: SceneBuilder| {
                        c.entity(tab_content_entity).despawn_related::<Children>();
                        c.ui_builder(tab_content_entity)
                            .spawn_scene_simple(("main", "main_tab"), &mut s);
                    },
                );
                // Set this up as the starting tab by selecting it
                let id = scene_handle.id();
                scene_handle.react().entity_event(id, Select);
            });

            sh.edit("tab_menu::settings", |scene_handle| {
                scene_handle.on_select(
                    move |mut c: Commands, mut s: SceneBuilder| {
                        c.entity(tab_content_entity).despawn_related::<Children>();
                        c.ui_builder(tab_content_entity).spawn_scene(
                            ("main", "settings_tab"),
                            &mut s,
                            |sh| {
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
                            },
                        );
                    },
                );
            });

            // set up exit tab
            sh.edit("tab_menu::exit", |scene_handle| {
                scene_handle.on_select(
                    move |mut c: Commands, mut s: SceneBuilder| {
                        c.entity(tab_content_entity).despawn_related::<Children>();
                        c.ui_builder(tab_content_entity)
                            .spawn_scene_simple(("main", "exit_tab"), &mut s);
                    },
                );
            });
        });
}

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, CobwebUiPlugin))
        .add_plugins(loading_screen_plugin)
        .load("manifest.cob")
        .add_systems(OnEnter(LoadState::Done), setup_ui);
}
