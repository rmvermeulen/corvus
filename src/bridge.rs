use crate::fs::{FsCommand, FsEvent};
use crate::prelude::*;
use crate::ui::ExplorerCommand;

#[derive(Clone, Debug, Deref, DerefMut, Event)]
pub struct DirectoryChangeRequest(PathBuf);

impl<P: Into<PathBuf>> From<P> for DirectoryChangeRequest {
    fn from(value: P) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Copy, Debug, Default, Event)]
pub struct CurrentDirectoryChanged;

fn forward_directory_changed(
    mut fs_events: EventReader<FsEvent>,
    mut current_directory_changed: EventWriter<CurrentDirectoryChanged>,
) {
    if fs_events
        .read()
        .filter(|e| matches!(e, FsEvent::DirectoryChanged(_)))
        .last()
        .is_some()
    {
        current_directory_changed.write_default();
    }
}

fn forward_not_a_directory(mut commands: Commands, mut fs_events: EventReader<FsEvent>) {
    if let Some(FsEvent::NotADirectory(path)) = fs_events
        .read()
        .filter(|e| matches!(e, FsEvent::NotADirectory(_)))
        .last()
    {
        commands
            .react()
            .broadcast(ExplorerCommand::SetPreview(Some(path.clone())));
    }
}

fn forward_directory_change_request(
    mut reader: EventReader<DirectoryChangeRequest>,
    mut writer: EventWriter<FsCommand>,
) {
    for DirectoryChangeRequest(event) in reader.read() {
        writer.write(FsCommand::ChangeDirectory(event.clone()));
    }
}

/// connects [fs_plugin] and [ui_plugin]
pub fn bridge_plugin(app: &mut App) {
    app.add_event::<DirectoryChangeRequest>()
        .add_event::<CurrentDirectoryChanged>()
        .add_systems(
            FixedUpdate,
            (
                forward_directory_change_request,
                forward_not_a_directory,
                forward_directory_changed,
            ),
        );
}
