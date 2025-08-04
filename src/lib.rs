use crate::explorer::explorer_plugin;
use crate::prelude::*;
use crate::resources::{LocationHistory, PreviewPath};

mod explorer;
mod fsio;
mod loading_screen;
mod resources;
mod traits;

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

pub fn root_plugin(app: &mut App) {
    app.add_plugins((DefaultPlugins, explorer_plugin));
}
