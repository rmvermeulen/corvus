use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

use cobble::{get_cobweb_entries, get_cobweb_manifest};

fn configure_hooks_path(path: &str) -> io::Result<bool> {
    Command::new("git")
        .args(["config", "--local", "core.hooksPath", path])
        .status()
        .map(|code| code.success())
}

fn update_manifest_if_changed<P: Into<PathBuf>>(
    manifest_path: P,
    manifest: String,
) -> io::Result<()> {
    let manifest_path = manifest_path.into();
    let is_manifest_up_to_date = fs::read_to_string(&manifest_path)
        .is_ok_and(|current_manifest| current_manifest == manifest);

    if is_manifest_up_to_date {
        Ok(())
    } else {
        fs::write(&manifest_path, manifest.as_bytes())
    }
}

fn main() {
    assert!(
        configure_hooks_path("hooks").unwrap(),
        "configure_hooks_path: unexpected exit code"
    );

    let cobweb_entries =
        get_cobweb_entries("assets/cobweb/", None).expect("get_cobweb_entries failed");

    // TODO: generate embed-macros for manifest files in release mode
    // for e in cobweb_entries { bevy_cobweb_UI::load_embedded_scene_file }

    let manifest = get_cobweb_manifest(cobweb_entries);
    update_manifest_if_changed("assets/cobweb/manifest.cob", manifest)
        .expect("update_cobweb_manifest failed");
}
