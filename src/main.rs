use bevy::app::App;
use cobweb_starter::explorer_plugin;

fn main() {
    App::new().add_plugins(explorer_plugin).run();
}
