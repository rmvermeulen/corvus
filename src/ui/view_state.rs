use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(CobwebLoadState = CobwebLoadState::Done)]
pub enum ViewState {
    /// Use this state to clear remaining persistent structures
    #[default]
    Unstable,
    /// Use this state to scope entities etc.
    Stable,
}

pub fn view_state_plugin(app: &mut App) {
    app.add_sub_state::<ViewState>().add_systems(
        OnEnter(ViewState::Unstable),
        |mut next_state: ResMut<NextState<ViewState>>| {
            debug!("Reset! going back to ViewState::Stable");
            next_state.set(ViewState::Stable);
        },
    );
}
