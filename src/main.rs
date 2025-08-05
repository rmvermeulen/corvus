use bevy::app::App;
use corvus::explorer_plugin;

fn main() {
    App::new().add_plugins(explorer_plugin).run();
}
