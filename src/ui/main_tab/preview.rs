use std::fs::read_to_string;
use std::io;
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css;
use bevy::image::{CompressedImageFormats, ImageType};

use crate::prelude::*;
use crate::resources::PreviewPath;

#[derive(Clone, Copy, Debug)]
enum PreviewMode<'a> {
    Text,
    Image(&'a str),
}

#[derive(Debug, thiserror::Error)]
enum ImageError {
    #[error("io::Error: {0}")]
    Io(#[from] io::Error),
    #[error("TextureError: {0}")]
    Texture(#[from] TextureError),
}

fn read_as_text<'a, P: AsRef<Path>>(path: P, mut builder: UiBuilder<'a, Entity>) -> io::Result<()> {
    read_to_string(path).map(move |text| {
        // TODO: wrap/no-wrap mode for text
        // TODO: slider for font-size
        builder.spawn((
            TextLayout::default().with_linebreak(LineBreak::NoWrap),
            Text::new(text),
        ));
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

fn on_new_preview_path<P: AsRef<Path>>(
    id: TargetId,
    commands: &mut Commands,
    images: &mut Assets<Image>,
    path: P,
) {
    let path = path.as_ref();
    let preview_mode = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| matches!(ext, "png" | "jpg" | "webp").then_some(PreviewMode::Image(ext)))
        .unwrap_or(PreviewMode::Text);

    let builder = commands.ui_builder(*id);

    match preview_mode {
        PreviewMode::Text => read_as_text(path, builder).map_err(Into::into),
        PreviewMode::Image(ext) => read_as_image(path, ext, images, builder),
    }
    // TODO: .or_else(|_| read_as_binary(path, commands.ui_builder(*id)))
    .unwrap_or_else(move |error| {
        commands
            .ui_builder(*id)
            .spawn((Text::new(format!("{error}")), TextColor::from(css::RED)));
    });
}

pub fn update_preview(
    id: TargetId,
    mut commands: Commands,
    preview_path: Res<PreviewPath>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("content::preview {preview_path:?}");
    // clear whatever we have now
    commands.entity(*id).despawn_related::<Children>();
    // build new preview, if required
    if let Some(path) = (*preview_path).as_ref() {
        on_new_preview_path(id, &mut commands, &mut images, path);
    }
}
