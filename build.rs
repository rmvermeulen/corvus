use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

use cobble::generate_cobweb_manifest;

fn configure_hooks_path(path: &str) -> io::Result<bool> {
    Command::new("git")
        .args(["config", "--local", "core.hooksPath", path])
        .status()
        .map(|code| code.success())
}

fn update_cobweb_manifest<P: Into<PathBuf>>(manifest_path: P) -> io::Result<()> {
    let manifest_path = manifest_path.into();
    let manifest = generate_cobweb_manifest(&manifest_path);
    if let Ok(current_manifest) = fs::read_to_string(&manifest_path)
        && current_manifest == manifest
    {
        Ok(())
    } else {
        fs::write(&manifest_path, manifest.as_bytes())
    }
}

// TODO: generate embed-macros for manifest files in release mode

fn main() {
    assert!(
        configure_hooks_path("hooks").unwrap(),
        "configure_hooks_path: unexpected exit code"
    );
    update_cobweb_manifest("assets/cobweb/manifest.cob").expect("update_cobweb_manifest failed");
}
