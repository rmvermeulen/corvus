use bevy::color::palettes::css::BLACK;

use crate::prelude::*;

#[derive(Component)]
struct LoadingScreen;

fn setup_loading_screen(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands
        .spawn((
            LoadingScreen,
            BackgroundColor::from(BLACK),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .apply(Padding::splat(Val::Px(15.0)))
        .apply(Margin::splat(Val::Auto))
        .apply(SetJustifyLines(JustifyLines::Center))
        .with_child((
            Text::new("Loading..."),
            TextColor::WHITE,
            TextFont::from_font_size(25.0),
        ));
}

fn cleanup_loading_screen(
    time: Res<Time>,
    mut commands: Commands,
    background_color: Single<(Entity, &mut BackgroundColor, &Children), With<LoadingScreen>>,
    mut text_colors: Query<&mut TextColor>,
) {
    let (e, mut bgc, children) = background_color.into_inner();
    let alpha = bgc.0.alpha() - time.delta_secs();
    if alpha <= 0.0 {
        commands.entity(e).despawn();
        debug!("despawned LoadingScreen");
    } else {
        bgc.0.set_alpha(alpha.max(0.0));
        for &child in children {
            if let Ok(mut text_color) = text_colors.get_mut(child) {
                text_color.set_alpha(alpha);
            }
        }
    }
}

pub(crate) fn loading_screen_plugin(app: &mut App) {
    app.add_systems(Startup, setup_loading_screen).add_systems(
        FixedUpdate,
        cleanup_loading_screen.run_if(any_match_filter::<With<LoadingScreen>>),
    );
}
