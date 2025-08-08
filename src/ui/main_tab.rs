use std::env;
use std::ffi::OsStr;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use itertools::Itertools;

use crate::config::ICON_CONFIG;
use crate::fs::EntryType;
use crate::resources::{CurrentDirectory, DirectoryEntries};
use crate::traits::{PathChecksExt, WithUiIcon};
use crate::ui::ui_events::{LocationSelectionUpdated, UpdateLocationText, UpdateOverview,
                           UpdatePreview};
use crate::ui::{ExplorerCommand, broadcast_fn};

pub mod preview;

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
            broadcast::<UpdateLocationText>(),
            |_: TargetId, mut commands: Commands, current_directory: Res<CurrentDirectory>| {
                let cwd = current_directory.to_string();
                commands
                    .react()
                    .broadcast(LocationSelectionUpdated::new_no_selection(cwd));
            },
        );

    fn update_text_fragment<'a>(
        mut handle: SceneHandle<'a, UiBuilder<'a, Entity>>,
        get_text: impl Fn(&LocationSelectionUpdated) -> &String + Send + Sync + 'static,
    ) {
        handle.update_on(
            broadcast::<LocationSelectionUpdated>(),
            move |id: TargetId,
                  broadcast_event: BroadcastEvent<LocationSelectionUpdated>,
                  mut text_editor: TextEditor| {
                if let Ok(ev) = broadcast_event.try_read() {
                    write_text!(text_editor, *id, "{}", get_text(ev));
                }
            },
        );
    }

    update_text_fragment(location.get("before"), |ev| &ev.before);
    update_text_fragment(location.get("selected"), |ev| &ev.selected);
    update_text_fragment(location.get("after"), |ev| &ev.after);

    location.update(|_: TargetId, mut commands: Commands| {
        commands.react().broadcast(UpdateLocationText);
    });
}

fn setup_navigation<'a>(navigation: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(navigation.path_ends_with(&["navigation"]));

    let configs = [
        (
            "back_button",
            ICON_CONFIG.navigation.back,
            ExplorerCommand::HistoryBack,
        ),
        (
            "next_button",
            ICON_CONFIG.navigation.next,
            ExplorerCommand::HistoryNext,
        ),
        (
            "up_button",
            ICON_CONFIG.navigation.up,
            ExplorerCommand::GotoParent,
        ),
        (
            "reload_button",
            ICON_CONFIG.navigation.reload,
            ExplorerCommand::Reload,
        ),
    ];

    for (name, icon, explorer_command) in configs {
        navigation
            .get(name)
            .update_text(icon)
            .on_pressed(move |mut commands: Commands| {
                let explorer_command = explorer_command.clone();
                info!("{explorer_command:?}");
                commands.react().broadcast(explorer_command);
            });
    }
    setup_location_text(&mut navigation.get("location"));
}

fn setup_header<'a>(header: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    assert!(header.path_ends_with(&["header"]));
    setup_navigation(&mut header.get("navigation"));
}

pub fn init_main_tab<'a>(sh: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
    info!("init_main_tab ({:?})", env::current_dir());
    setup_header(&mut sh.get("header"));

    sh.get("content::overview::items").update_on(
        broadcast::<UpdateOverview>(),
        |id: TargetId,
         mut commands: Commands,
         mut scene_builder: SceneBuilder,
         entries: Res<DirectoryEntries>| {
            info!("content::overview::items on broadcast UpdateOverview");
            commands.entity(*id).despawn_related::<Children>();

            let mut entries: Vec<_> = entries.clone();
            entries.sort();

            for entry in entries {
                let path = entry.path();
                let entry_type = entry.entry_type();
                let menu_command = match entry_type {
                    EntryType::File => Some(ExplorerCommand::SetPreview(Some(path.to_owned()))),
                    EntryType::Directory | EntryType::Symlink => {
                        Some(ExplorerCommand::SetDirectory(path.to_owned()))
                    }
                    _ => None,
                };
                let mut builder = commands.ui_builder(*id);
                // spawn icon button
                builder.spawn_scene(("widgets", "button"), &mut scene_builder, |icon_button| {
                    icon_button.on_pressed(|| {
                        info!("overview-item[icon]: on_pressed not implemented!");
                    });
                    icon_button.get("text").update_text(entry_type.get_icon());
                });
                // spawn text button (filename)
                builder.spawn_scene(
                    ("widgets", "button"),
                    &mut scene_builder,
                    |filename_button| {
                        filename_button.insert(entry_type);
                        if let Some(menu_command) = menu_command {
                            filename_button.on_pressed(broadcast_fn(menu_command));
                        }
                        if let Some(name) = path.file_stem().map(OsStr::to_string_lossy) {
                            filename_button.get("text").update_text(name);
                        } else {
                            // text still impacts width
                            filename_button.get("text").update_text("");
                            filename_button.insert(Visibility::Hidden);
                        }
                    },
                );
                // spawn text button (extension)
                builder.spawn_scene(("widgets", "button"), &mut scene_builder, |ext_button| {
                    ext_button.on_pressed(|| {
                        info!("overview-item[ext]: on_pressed not implemented!");
                    });
                    if let Some(ext) = path.extension().map(OsStr::to_string_lossy) {
                        ext_button.get("text").update_text(ext);
                    } else {
                        // text still impacts width
                        ext_button.get("text").update_text("");
                        ext_button.insert(Visibility::Hidden);
                    }
                });
            }
        },
    );

    sh.get("content::preview::scroll::view_shim::view::shim")
        .update_on(broadcast::<UpdatePreview>(), preview::update_preview);
}
