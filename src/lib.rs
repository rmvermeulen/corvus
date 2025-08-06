use crate::fs::fs_plugin;
use crate::prelude::*;
use crate::resources::{LocationHistory, PreviewPath};
use crate::ui::ui_plugin;

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

pub fn explorer_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, fs_plugin, ui_plugin));
}
