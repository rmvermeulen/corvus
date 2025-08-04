use crate::explorer::{AppCommand, AppTab};
use crate::prelude::*;

pub trait PathChecksExt {
    fn path_vec(&self) -> Vec<&str>;
    fn path_ends_with(&self, needle: &[&str]) -> bool {
        self.path_vec().ends_with(needle)
    }
}

impl<'a, B: scene_traits::SceneNodeBuilderOuter<'a>> PathChecksExt for SceneHandle<'a, B> {
    fn path_vec(&self) -> Vec<&str> {
        self.path().path.iter().collect_vec()
    }
}

pub trait ChangeTabExt {
    fn change_tab(&mut self, tab: AppTab);
}

impl<'w, 's> ChangeTabExt for ReactCommands<'w, 's> {
    fn change_tab(&mut self, tab: AppTab) {
        self.broadcast(AppCommand::ChangeTab(tab));
    }
}

impl<'w, 's> ChangeTabExt for Commands<'w, 's> {
    fn change_tab(&mut self, tab: AppTab) {
        self.react().change_tab(tab);
    }
}
