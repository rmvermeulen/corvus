use std::fs::{self, DirEntry};
use std::io;
use std::process::Command;

use itertools::Itertools;
use rayon::prelude::*;

fn generate_cobweb_manifest(manifest_path: &str) -> std::io::Result<()> {
    let current_manifest = fs::read_to_string(manifest_path).expect("Failed to read manifest.cob");

    let names = fs::read_dir("assets")?
        .collect::<Vec<_>>()
        .into_par_iter()
        .filter_map(|res| res.ok())
        .filter(|entry: &DirEntry| {
            entry.file_type().is_ok_and(|ft| ft.is_file())
                && entry.path().extension().is_some_and(|ext| ext == "cob")
        })
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let name = name.strip_suffix(".cob")?;
            // do not include the manifest inside itself
            (name != "manifest").then(|| name.to_string())
        })
        .collect::<Vec<_>>();
    let manifest = ["#manifest".to_string()]
        .into_iter()
        .chain(
            names
                .into_iter()
                .sorted()
                .map(|name| format!(r#""{name}.cob" as {name}"#)),
        )
        .join("\n");
    if current_manifest == manifest {
        // nothing changed
        Ok(())
    } else {
        fs::write("assets/manifest.cob", manifest.as_bytes())
    }
}

fn configure_hooks_path(path: &str) -> io::Result<bool> {
    Command::new("git")
        .args(["config", "--local", "core.hooksPath", path])
        .status()
        .map(|code| code.success())
}

fn main() {
    assert!(
        configure_hooks_path("hooks").unwrap(),
        "configure_hooks_path: unexpected exit code"
    );
    generate_cobweb_manifest("assets/manifest.cob").expect("generate_cobweb_manifest failed");
}
