use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEntry {
    path: PathBuf,
    name: String,
}

impl ManifestEntry {
    fn new<P: Into<PathBuf>, S: ToString>(path: P, name: S) -> Self {
        Self {
            path: path.into(),
            name: name.to_string(),
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialOrd for ManifestEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for ManifestEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

fn process_dir(entry: DirEntry, prefix: Option<String>) -> io::Result<Vec<ManifestEntry>> {
    let dir_name = entry
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let prefix = prefix
        .map(|prefix| format!("{prefix}_{dir_name}"))
        .unwrap_or(dir_name);
    get_cobweb_entries(entry.path(), Some(prefix))
}

fn process_file(name: &OsStr, entry: DirEntry, prefix: Option<String>) -> ManifestEntry {
    let path = entry.path().iter().skip(1).collect::<PathBuf>();
    let name = name.to_string_lossy();
    let name = prefix
        .map(|prefix| format!("{prefix}_{name}"))
        .unwrap_or_else(|| name.to_string());
    ManifestEntry::new(path, name)
}

fn process_entry(
    name: &OsStr,
    entry: DirEntry,
    prefix: Option<String>,
) -> io::Result<Vec<ManifestEntry>> {
    let ft = entry.file_type()?;
    if ft.is_file() && entry.path().extension().is_some_and(|ext| ext == "cob") {
        let entry = process_file(name, entry, prefix);
        Ok(vec![entry])
    } else if ft.is_dir() {
        process_dir(entry, prefix)
    } else {
        Ok(Vec::default())
    }
}

pub fn get_cobweb_entries<P: Into<PathBuf>>(
    root_dir: P,
    prefix: Option<String>,
) -> io::Result<Vec<ManifestEntry>> {
    let root_dir: PathBuf = root_dir.into();
    let entries = fs::read_dir(root_dir)?
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            let name = path.file_stem().unwrap_or_default();
            process_entry(name, entry, prefix.clone())
        })
        .flatten()
        .collect::<Vec<ManifestEntry>>();
    Ok(entries)
}

pub fn get_cobweb_manifest<E: IntoIterator<Item = ManifestEntry>>(entries: E) -> String {
    let mut entries = entries
        .into_iter()
        // do not include the manifest inside itself
        .filter(|entry| entry.name != "manifest")
        .collect::<Vec<_>>();

    entries.sort();
    let imports = entries
        .into_iter()
        .map(|ManifestEntry { path, name }| format!(r#""{}" as {name}"#, path.to_string_lossy()));

    ["#manifest".to_string()]
        .into_iter()
        .chain(imports)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn generate_cobweb_manifest<P: Into<PathBuf>>(manifest_path: P) -> io::Result<String> {
    let manifest_path: PathBuf = manifest_path.into();
    get_cobweb_entries(
        manifest_path.parent().expect("manifest must be in assets/"),
        None,
    )
    .map(get_cobweb_manifest)
}
