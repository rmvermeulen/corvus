use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::UpdateTextExt;

use crate::PanelLayout;

pub fn init_settings_tab<'a>(settings_tab: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
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
