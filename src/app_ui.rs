use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use itertools::Itertools;

use crate::ui_events::CurrentDirectoryChanged;
use crate::view_state::ViewState;
use crate::{ActiveTab, AppCommand, AppTab, ChangeTabExt, DespawnUi, ExplorerCommand,
            PathChecksExt, broadcast_fn, main_tab, settings_tab,
            update_explorer_on_explorer_command};

pub fn update_tab_content_on_app_command(
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
                    // show the current directory
                    commands.react().broadcast(CurrentDirectoryChanged);
                }
                AppTab::Settings => {
                    commands.ui_builder(id).spawn_scene(
                        ("tabs_settings", "settings_tab"),
                        &mut scene_builder,
                        settings_tab::init_settings_tab,
                    );
                }
            }
            commands.insert_resource(ActiveTab(*tab));
        }
    }
}

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

pub fn build_ui(
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
