use std::env;
use std::fs::read_to_string;
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css;
use bevy::image::{CompressedImageFormats, ImageType};
use bevy::prelude::*;
use bevy::tasks::futures_lite::io;
use bevy::ui::RelativeCursorPosition;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use itertools::Itertools;

use crate::explorer::ui_events::{CurrentDirectoryChanged, LocationSelectionUpdated,
                                 PreviewPathChanged};
use crate::explorer::{ExplorerCommand, broadcast_fn};
use crate::fsio::EntryType;
use crate::resources::{CurrentDirectory, PreviewPath};
use crate::traits::PathChecksExt;

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
            broadcast::<CurrentDirectoryChanged>(),
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

    #[derive(Clone, Copy, Debug)]
    enum PreviewMode<'a> {
        Text,
        Image(&'a str),
        // TODO: Binary,
    }

    sh.get("content::preview").update_on(
        broadcast::<PreviewPathChanged>(),
        |id: TargetId,
         mut commands: Commands,
         preview_path: Res<PreviewPath>,
         mut images: ResMut<Assets<Image>>| {
            commands.entity(*id).despawn_related::<Children>();
            info!("{preview_path:?}");
            if let Some(path) = (*preview_path).as_ref() {
                let preview_mode = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| {
                        matches!(ext, "png" | "jpg" | "webp").then_some(PreviewMode::Image(ext))
                    })
                    .unwrap_or(PreviewMode::Text);

                #[derive(Debug, thiserror::Error)]
                enum ImageError {
                    #[error("io::Error: {0}")]
                    Io(#[from] io::Error),
                    #[error("TextureError: {0}")]
                    Texture(#[from] TextureError),
                }

                fn read_as_text<'a, P: AsRef<Path>>(
                    path: P,
                    mut builder: UiBuilder<'a, Entity>,
                ) -> io::Result<()> {
                    read_to_string(path).map(move |text| {
                        builder.spawn(Text::new(text));
                    })
                }

                fn read_as_image<'a, P: AsRef<Path>>(
                    path: P,
                    ext: &str,
                    images: &mut Assets<Image>,
                    mut builder: UiBuilder<'a, Entity>,
                ) -> Result<(), ImageError> {
                    std::fs::read(path)
                        .map_err(ImageError::from)
                        .and_then(|bytes| {
                            Image::from_buffer(
                                &bytes,
                                ImageType::Extension(ext),
                                CompressedImageFormats::default(),
                                false,
                                bevy::image::ImageSampler::Default,
                                RenderAssetUsages::RENDER_WORLD,
                            )
                            .map_err(ImageError::from)
                        })
                        .map(move |image| {
                            let image = images.add(image);
                            builder.spawn(ImageNode::new(image));
                        })
                }

                fn read_as_binary<'a, P: AsRef<Path>>(
                    _path: P,
                    mut _builder: UiBuilder<'a, Entity>,
                ) -> Result<(), ImageError> {
                    todo!("read_as_binary using xxd")
                }

                match preview_mode {
                    PreviewMode::Text => {
                        read_as_text(path, commands.ui_builder(*id)).map_err(Into::into)
                    }
                    PreviewMode::Image(ext) => {
                        read_as_image(path, ext, &mut images, commands.ui_builder(*id))
                    }
                }
                // TODO: .or_else(|_| read_as_binary(path, commands.ui_builder(*id)))
                .unwrap_or_else(move |error| {
                    commands
                        .ui_builder(*id)
                        .spawn((Text::new(format!("{error}")), TextColor::from(css::RED)));
                });
            }
        },
    );
}
