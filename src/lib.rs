use crate::bridge::bridge_plugin;
use crate::fs::fs_plugin;
use crate::prelude::*;
use crate::resources::{LocationHistory, PreviewPath};
use crate::ui::ui_plugin;

mod bridge;
#[cfg(debug_assertions)]
mod cobweb_warning_subscriber;
pub mod config;
mod fs;
mod resources;
mod traits;
mod ui;

mod prelude {
    pub use std::path::{Path, PathBuf};
    pub type CobwebLoadState = bevy_cobweb_ui::prelude::LoadState;

    pub use bevy::prelude::*;
    pub use bevy_cobweb::prelude::*;
    pub use bevy_cobweb_ui::prelude::*;
    pub use cfg_if::cfg_if;
    pub use itertools::Itertools;
}

/// example: inspect current nodes
/// ```rust no_run
/// fn inspect_handle<'a>(handle: &mut SceneHandle<'a, UiBuilder<'a, Entity>>) {
///     fn on_inspect(id: TargetId, mut text_editor: TextEditor, children: Query<&Children>) {
///         for (i, child) in children.iter_descendants(*id).enumerate() {
///             // look up components etc
///         }
///     }
///     assert_eq!(sh.path(), "scroll_list::options::view::shim");
///     handle.update(on_inspect);
/// }
///```

pub fn corvus_plugin(app: &mut App) {
    cfg_if! {
        if #[cfg(debug_assertions)] {
            let log_plugin =cobweb_warning_subscriber:: get_log_plugin();
        } else {
            let log_plugin = LogPlugin::default();
        }
    }
    app.add_plugins(DefaultPlugins.set(log_plugin))
        .add_plugins(bridge_plugin)
        .add_plugins((fs_plugin, ui_plugin));
}
