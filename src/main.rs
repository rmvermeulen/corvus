use bevy::app::App;
use cobweb_starter::root_plugin;

fn main() {
    App::new().add_plugins(root_plugin).run();
}
