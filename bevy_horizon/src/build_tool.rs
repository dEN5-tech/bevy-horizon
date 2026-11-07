use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Horizon mod compatibility descriptor matching `compatibility.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompatibilityJson {
    pub game: String,
    pub hooks: u32,
    #[serde(default)]
    pub horizon_repository: String,
    pub horizon_revision: String,
    #[serde(default)]
    pub horizon_mixins_repository: String,
    pub horizon_mixins_revision: String,
}

/// Metadata header in `horizon-native.json` inside the `.hnmod` archive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HorizonNativeManifest {
    pub game: String,
    pub hooks: u32,
    pub horizon: String,
    pub id: String,
    pub mixins: String,
    pub schema: u32,
    pub version: String,
}

/// Summary result of the `.hnmod` packaging process.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageResult {
    pub out_path: PathBuf,
    pub file_count: usize,
    pub size_bytes: u64,
}

/// Helper to recursively gather all `.rs` and `.toml` files under a directory.
fn gather_files(dir: &Path, base: &Path, list: &mut Vec<(String, Vec<u8>)>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            gather_files(&path, base, list)?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "rs" || ext == "toml" {
                let rel = path.strip_prefix(base).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                let data = fs::read(&path)?;
                list.push((rel_str, data));
            }
        }
    }
    Ok(())
}

/// Packages a Horizon native mod directory into a deterministic `.hnmod` zip archive.
///
/// * `manifest_or_compat_path` - Path to `compatibility.json` or `manifest.json`.
/// * `mixins_bundle_dir` - Path to the mod's mixin folder (e.g. `mixins/<mod_id>`).
/// * `out_path` - Destination file path for `<mod_id>.hnmod`.
pub fn package_hnmod(
    manifest_or_compat_path: impl AsRef<Path>,
    mixins_bundle_dir: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
) -> io::Result<PackageResult> {
    let compat_path = manifest_or_compat_path.as_ref();
    let bundle_dir = mixins_bundle_dir.as_ref();
    let out_file_path = out_path.as_ref();

    if !compat_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Manifest or compatibility file not found: {}", compat_path.display()),
        ));
    }

    if !bundle_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Mixins bundle directory not found: {}", bundle_dir.display()),
        ));
    }

    let compat_bytes = fs::read(compat_path)?;
    let compat: CompatibilityJson = serde_json::from_slice(&compat_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse compatibility JSON: {e}")))?;

    // Extract mod ID from directory name or mixin manifest
    let mod_id = bundle_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mod")
        .to_string();

    let manifest = HorizonNativeManifest {
        game: compat.game,
        hooks: compat.hooks,
        horizon: compat.horizon_revision,
        id: mod_id.clone(),
        mixins: compat.horizon_mixins_revision,
        schema: 1,
        version: "0.1.0".to_string(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        + "\n";

    if let Some(parent) = out_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = fs::File::create(out_file_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. Write horizon-native.json
    zip.start_file("horizon-native.json", options)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zip.write_all(manifest_json.as_bytes())?;

    // 2. Gather and write mixins/<mod_id> files
    let mut file_entries = Vec::new();
    gather_files(bundle_dir, bundle_dir, &mut file_entries)?;
    file_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let count = file_entries.len() + 1;
    for (rel_path, content) in file_entries {
        let zip_path = format!("mixins/{mod_id}/{rel_path}");
        zip.start_file(zip_path, options)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        zip.write_all(&content)?;
    }

    // 3. Check for LICENSE in parent of mixins
    let potential_license = bundle_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("LICENSE"));

    if let Some(license_path) = potential_license {
        if license_path.is_file() {
            let license_bytes = fs::read(&license_path)?;
            let zip_path = format!("mixins/{mod_id}/LICENSE");
            zip.start_file(zip_path, options)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            zip.write_all(&license_bytes)?;
        }
    }

    zip.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let meta = fs::metadata(out_file_path)?;
    Ok(PackageResult {
        out_path: out_file_path.to_path_buf(),
        file_count: count,
        size_bytes: meta.len(),
    })
}
