use bevy::utils::default;

#[derive(Clone, Copy, Debug, Default)]
pub struct PreviewPathChanged;

#[derive(Clone, Copy, Debug, Default)]
pub struct CurrentDirectoryChanged;

#[derive(Clone, Debug, Default)]
pub struct LocationSelectionUpdated {
    pub before: String,
    pub selected: String,
    pub after: String,
}

impl LocationSelectionUpdated {
    pub fn new_no_selection(text: String) -> Self {
        Self {
            before: text,
            selected: default(),
            after: default(),
        }
    }
    pub fn new(before: String, selected: String, after: String) -> Self {
        Self {
            before,
            selected,
            after,
        }
    }
}

