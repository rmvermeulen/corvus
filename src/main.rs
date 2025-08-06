use bevy::app::App;
use corvus::corvus_plugin;

fn main() {
    App::new().add_plugins(corvus_plugin).run();
}
