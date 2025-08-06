use std::env;
use std::ffi::OsStr;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use itertools::Itertools;

use crate::fs::EntryType;
use crate::resources::CurrentDirectory;
use crate::traits::PathChecksExt;
use crate::ui::ui_events::{LocationSelectionUpdated, UpdateLocationText, UpdatePreview};
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
        ("back_button", ExplorerCommand::HistoryBack),
        ("next_button", ExplorerCommand::HistoryNext),
        ("up_button", ExplorerCommand::GotoParent),
        ("reload_button", ExplorerCommand::Reload),
    ];

    for (name, explorer_command) in configs {
        navigation
            .get(name)
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

    // TODO: async io (queue?)
    // TODO: independent of ui
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
                sh.on_pressed(|| {
                    info!("overview-item[icon]: on_pressed not implemented!");
                });
                let icon = entry_type.get_icon();
                sh.get("text").update_text(format!("[{icon}]"));
            })
            .spawn_scene(("widgets", "button"), |sh| {
                sh.insert(entry_type);
                if let Some(menu_command) = menu_command {
                    sh.on_pressed(broadcast_fn(menu_command));
                }
                if let Some(name) = path.file_stem().map(OsStr::to_string_lossy) {
                    sh.get("text").update_text(name);
                } else {
                    // text still impacts width
                    sh.get("text").update_text("");
                    sh.insert(Visibility::Hidden);
                }
            })
            .spawn_scene(("widgets", "button"), |sh| {
                sh.on_pressed(|| {
                    info!("overview-item[ext]: on_pressed not implemented!");
                });
                if let Some(ext) = path.extension().map(OsStr::to_string_lossy) {
                    sh.get("text").update_text(ext);
                } else {
                    // text still impacts width
                    sh.get("text").update_text("");
                    sh.insert(Visibility::Hidden);
                }
            });
    }

    sh.get("content::preview::scroll::view_shim::view::shim")
        .update_on(broadcast::<UpdatePreview>(), preview::update_preview);
}
