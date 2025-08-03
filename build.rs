use std::fs::DirEntry;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

type Line = (String, String);

fn cobble<P: Into<PathBuf>>(path: P, prefix: Option<String>) -> impl IntoIterator<Item = Line> {
    let path: PathBuf = path.into();
    if path.is_file() {
        return Vec::new();
    }

    fn process_dir(entry: DirEntry, prefix: Option<String>) -> impl IntoIterator<Item = Line> {
        let dir_name = entry
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let prefix = prefix
            .map(|prefix| format!("{prefix}_{dir_name}"))
            .unwrap_or(dir_name);
        cobble(entry.path(), Some(prefix))
    }

    fn process_file(name: &std::ffi::OsStr, entry: DirEntry, prefix: Option<String>) -> Line {
        let path = entry
            .path()
            .iter()
            .skip(1)
            .collect::<PathBuf>()
            .to_string_lossy()
            .to_string();
        let name = name.to_string_lossy().to_string();
        let name = prefix
            .map(|prefix| format!("{prefix}_{name}"))
            .unwrap_or(name);
        (path, name)
    }

    fn process_entry(
        name: &std::ffi::OsStr,
        entry: fs::DirEntry,
        prefix: Option<String>,
    ) -> Option<Vec<Line>> {
        let ft = entry.file_type().ok()?;
        if ft.is_file() && entry.path().extension().is_some_and(|ext| ext == "cob") {
            Some(vec![process_file(name, entry, prefix)])
        } else if ft.is_dir() {
            let x = process_dir(entry, prefix);
            Some(x.into_iter().collect())
        } else {
            None
        }
    }

    let entries = fs::read_dir(path)
        .map(|entries| entries.flatten().collect())
        .unwrap_or_else(|_| Vec::new());

    entries
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_stem()?;
            process_entry(name, entry, prefix.clone())
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn generate_cobweb_manifest<P: Into<PathBuf>>(manifest_path: P) -> std::io::Result<()> {
    let manifest_path: PathBuf = manifest_path.into();
    let entries = cobble(
        manifest_path.parent().expect("manifest must be in assets/"),
        None,
    )
    .into_iter()
    .collect::<Vec<_>>();

    let mut names_to_import = entries
        .into_iter()
        // do not include the manifest inside itself
        .filter(|(_, name)| name != "manifest")
        .collect::<Vec<_>>();

    names_to_import.sort();
    let imports = names_to_import
        .into_iter()
        .map(|(path, name)| format!(r#""{path}" as {name}"#));

    let manifest = ["#manifest".to_string()]
        .into_iter()
        .chain(imports)
        .collect::<Vec<_>>()
        .join("\n");
    if let Ok(current_manifest) = fs::read_to_string(&manifest_path)
        && current_manifest == manifest
    {
        // nothing changed
        Ok(())
    } else {
        fs::write(&manifest_path, manifest.as_bytes())
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
    generate_cobweb_manifest("assets/cobweb/manifest.cob")
        .expect("generate_cobweb_manifest failed");
}
