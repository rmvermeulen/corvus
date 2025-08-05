use std::env;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use itertools::Itertools;

use crate::fs::EntryType;
use crate::resources::CurrentDirectory;
use crate::traits::PathChecksExt;
use crate::ui::ui_events::{LocationSelectionUpdated, UpdateCurrentDirectory, UpdatePreview};
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
            broadcast::<UpdateCurrentDirectory>(),
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
                sh.insert(entry_type);
                if let Some(menu_command) = menu_command {
                    sh.on_pressed(broadcast_fn(menu_command));
                }
                let label = path.to_string_lossy();
                sh.get("text")
                    .update_text(format!("[{}] {label}", entry_type.get_char()));
            });
    }

    sh.get("content::preview::scroll::view_shim::view::shim")
        .update_on(broadcast::<UpdatePreview>(), preview::update_preview);
}
