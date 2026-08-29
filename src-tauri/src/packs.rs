use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

const MANIFEST_FILE: &str = "pet-pack.json";
const MANAGED_ROOT_MARKER: &str = ".virtual-pet-external-packs-v1";
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const REQUIRED_ANIMATIONS: [&str; 4] = ["idle", "walk", "run", "sleep"];

#[derive(Debug)]
pub struct PackError(String);

impl PackError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for PackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for PackError {}

impl From<io::Error> for PackError {
    fn from(error: io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<serde_json::Error> for PackError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetPackManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub species: String,
    pub description: String,
    pub authors: Vec<PackAuthor>,
    pub atlas: PackAtlas,
    pub animations: BTreeMap<String, PackAnimation>,
    #[serde(default)]
    pub sounds: BTreeMap<String, PackSound>,
    pub license: PackLicense,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PackAuthor {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackAtlas {
    pub file: String,
    pub columns: u32,
    pub rows: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackAnimation {
    pub row: u32,
    pub frames: Vec<u32>,
    pub frame_duration_ms: u32,
    #[serde(rename = "loop")]
    pub looping: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PackSound {
    pub file: String,
    pub volume: f64,
    #[serde(rename = "loop")]
    pub looping: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PackLicense {
    pub spdx: String,
    pub assets: Vec<PackAssetLicense>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PackAssetLicense {
    pub path: String,
    pub creator: String,
    pub source: String,
    pub spdx: String,
    pub modifications: String,
}

struct ValidatedPack {
    root: PathBuf,
    manifest: PetPackManifest,
    manifest_bytes: Vec<u8>,
    assets: BTreeSet<String>,
}

/// Validates an unpacked external pet pack without modifying it.
pub fn validate_pack(source: &Path) -> Result<PetPackManifest, PackError> {
    Ok(validate_pack_internal(source)?.manifest)
}

/// Installs a validated pack below the supplied app-managed packs directory.
/// Bundled resource directories should never be passed as `packs_root`.
pub fn install_pack(source: &Path, packs_root: &Path) -> Result<PetPackManifest, PackError> {
    let pack = validate_pack_internal(source)?;
    let root = managed_root(packs_root, true)?;
    initialize_managed_root(&root)?;
    let destination = root.join(&pack.manifest.id);
    if fs::symlink_metadata(&destination).is_ok() {
        return Err(PackError::new(format!(
            "pet pack {} is already installed",
            pack.manifest.id
        )));
    }

    let temporary = create_temporary_directory(&root, &pack.manifest.id)?;
    let install_result = (|| {
        let manifest_path = temporary.join(MANIFEST_FILE);
        let mut manifest_file = File::create(&manifest_path)?;
        manifest_file.write_all(&pack.manifest_bytes)?;
        manifest_file.sync_all()?;

        for relative in &pack.assets {
            let source_file = secure_file(&pack.root, relative)?;
            let target_file = temporary.join(relative);
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source_file, target_file)?;
        }

        if fs::symlink_metadata(&destination).is_ok() {
            return Err(PackError::new(format!(
                "pet pack {} was installed concurrently",
                pack.manifest.id
            )));
        }
        fs::rename(&temporary, &destination)?;
        Ok(())
    })();

    if let Err(error) = install_result {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    Ok(pack.manifest)
}

/// Lists valid external packs installed directly below the app-managed root.
pub fn list_installed_packs(packs_root: &Path) -> Result<Vec<PetPackManifest>, PackError> {
    if !packs_root.exists() {
        return Ok(Vec::new());
    }
    let root = managed_root(packs_root, false)?;
    require_managed_root(&root)?;
    let mut packs = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(".install-") {
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(PackError::new(format!(
                "installed pack entry {} is a symlink",
                entry.path().display()
            )));
        }
        if !metadata.is_dir() {
            continue;
        }
        let manifest = validate_pack(&entry.path())?;
        if entry.file_name().to_string_lossy() != manifest.id {
            return Err(PackError::new(format!(
                "installed directory name does not match pack id {}",
                manifest.id
            )));
        }
        packs.push(manifest);
    }
    packs.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(packs)
}

/// Removes exactly one validated id directly below the app-managed packs root.
pub fn remove_installed_pack(packs_root: &Path, id: &str) -> Result<(), PackError> {
    validate_pack_id(id)?;
    let root = managed_root(packs_root, false)?;
    require_managed_root(&root)?;
    let target = root.join(id);
    let metadata = fs::symlink_metadata(&target)
        .map_err(|error| PackError::new(format!("cannot remove pet pack {id}: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PackError::new(format!(
            "installed pet pack {id} is not a regular directory"
        )));
    }
    let canonical_target = fs::canonicalize(&target)?;
    if canonical_target.parent() != Some(root.as_path()) {
        return Err(PackError::new(format!(
            "installed pet pack {id} is outside the managed packs directory"
        )));
    }
    fs::remove_dir_all(canonical_target)?;
    Ok(())
}

fn validate_pack_internal(source: &Path) -> Result<ValidatedPack, PackError> {
    let source_metadata = fs::symlink_metadata(source)
        .map_err(|error| PackError::new(format!("cannot access pack directory: {error}")))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err(PackError::new(
            "pack source must be a regular directory, not a symlink",
        ));
    }
    let root = fs::canonicalize(source)?;
    let manifest_path = secure_file(&root, MANIFEST_FILE)?;
    let manifest_size = fs::metadata(&manifest_path)?.len();
    if manifest_size > MAX_MANIFEST_BYTES {
        return Err(PackError::new("pet-pack.json exceeds the 1 MiB limit"));
    }
    let mut manifest_bytes = Vec::with_capacity(manifest_size as usize);
    File::open(manifest_path)?.read_to_end(&mut manifest_bytes)?;
    let manifest: PetPackManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| PackError::new(format!("invalid pet-pack.json: {error}")))?;
    let assets = validate_manifest(&manifest)?;

    for relative in &assets {
        secure_file(&root, relative)?;
    }
    let atlas_path = secure_file(&root, &manifest.atlas.file)?;
    validate_png_dimensions(&atlas_path, &manifest.atlas)?;

    Ok(ValidatedPack {
        root,
        manifest,
        manifest_bytes,
        assets,
    })
}

fn validate_manifest(manifest: &PetPackManifest) -> Result<BTreeSet<String>, PackError> {
    if manifest.schema_version != 1 {
        return Err(PackError::new("schemaVersion must be 1"));
    }
    validate_pack_id(&manifest.id)?;
    require_text(&manifest.name, "name")?;
    if !is_semantic_version(&manifest.version) {
        return Err(PackError::new("version must be a semantic version"));
    }
    if !is_lower_identifier(&manifest.species, true) {
        return Err(PackError::new("species must be a lowercase identifier"));
    }
    require_text(&manifest.description, "description")?;
    if manifest.authors.is_empty() {
        return Err(PackError::new("authors must not be empty"));
    }
    for author in &manifest.authors {
        require_text(&author.name, "author name")?;
    }

    safe_relative_path(&manifest.atlas.file)?;
    if extension(&manifest.atlas.file).as_deref() != Some("png") {
        return Err(PackError::new("atlas.file must be a PNG"));
    }
    if manifest.atlas.columns == 0
        || manifest.atlas.rows == 0
        || manifest.atlas.frame_width == 0
        || manifest.atlas.frame_height == 0
    {
        return Err(PackError::new(
            "atlas dimensions and grid values must be positive",
        ));
    }
    manifest
        .atlas
        .columns
        .checked_mul(manifest.atlas.frame_width)
        .ok_or_else(|| PackError::new("atlas width overflows"))?;
    manifest
        .atlas
        .rows
        .checked_mul(manifest.atlas.frame_height)
        .ok_or_else(|| PackError::new("atlas height overflows"))?;

    for required in REQUIRED_ANIMATIONS {
        if !manifest.animations.contains_key(required) {
            return Err(PackError::new(format!("animations.{required} is required")));
        }
    }
    for (name, animation) in &manifest.animations {
        if !is_action_name(name) {
            return Err(PackError::new(format!(
                "animation name {name:?} is invalid"
            )));
        }
        if animation.row >= manifest.atlas.rows {
            return Err(PackError::new(format!(
                "animations.{name}.row must reference an atlas row"
            )));
        }
        if animation.frames.is_empty()
            || animation
                .frames
                .iter()
                .any(|frame| *frame >= manifest.atlas.columns)
        {
            return Err(PackError::new(format!(
                "animations.{name}.frames must reference atlas columns"
            )));
        }
        if !(40..=60_000).contains(&animation.frame_duration_ms) {
            return Err(PackError::new(format!(
                "animations.{name}.frameDurationMs must be from 40 to 60000"
            )));
        }
    }

    let mut assets = BTreeSet::from([manifest.atlas.file.clone()]);
    for (name, sound) in &manifest.sounds {
        if !is_action_name(name) {
            return Err(PackError::new(format!("sound name {name:?} is invalid")));
        }
        safe_relative_path(&sound.file)?;
        if !matches!(
            extension(&sound.file).as_deref(),
            Some("wav" | "mp3" | "ogg" | "m4a")
        ) {
            return Err(PackError::new(format!(
                "sounds.{name}.file uses an unsupported audio extension"
            )));
        }
        if !sound.volume.is_finite() || !(0.0..=1.0).contains(&sound.volume) {
            return Err(PackError::new(format!(
                "sounds.{name}.volume must be between 0 and 1"
            )));
        }
        assets.insert(sound.file.clone());
    }

    require_text(&manifest.license.spdx, "license.spdx")?;
    let mut licensed_paths = BTreeSet::new();
    for asset in &manifest.license.assets {
        safe_relative_path(&asset.path)?;
        require_text(&asset.creator, "license asset creator")?;
        require_text(&asset.source, "license asset source")?;
        require_text(&asset.spdx, "license asset spdx")?;
        require_text(&asset.modifications, "license asset modifications")?;
        if !licensed_paths.insert(asset.path.clone()) {
            return Err(PackError::new(format!(
                "license asset path {} is duplicated",
                asset.path
            )));
        }
    }
    for asset in &assets {
        if !licensed_paths.contains(asset) {
            return Err(PackError::new(format!(
                "license.assets must document {asset}"
            )));
        }
    }
    Ok(assets)
}

fn validate_pack_id(id: &str) -> Result<(), PackError> {
    if !is_lower_identifier(id, false) {
        return Err(PackError::new(
            "id must be a lowercase reverse-domain-style identifier",
        ));
    }
    Ok(())
}

fn is_lower_identifier(value: &str, hyphen_only: bool) -> bool {
    if value.is_empty() || !value.is_ascii() {
        return false;
    }
    let mut separator_seen = hyphen_only;
    let mut previous_separator = false;
    for (index, byte) in value.bytes().enumerate() {
        let separator = byte == b'-' || (!hyphen_only && byte == b'.');
        if separator {
            if index == 0 || previous_separator {
                return false;
            }
            separator_seen = true;
        } else if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
            return false;
        }
        previous_separator = separator;
    }
    separator_seen && !previous_separator
}

fn is_action_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(first) if first.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_semantic_version(value: &str) -> bool {
    let (core, suffix) = value
        .split_once('-')
        .map_or((value, None), |(core, suffix)| (core, Some(suffix)));
    let mut parts = core.split('.');
    let valid_core = (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) && parts.next().is_none();
    valid_core
        && suffix.is_none_or(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-')
        })
}

fn require_text(value: &str, field: &str) -> Result<(), PackError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(PackError::new(format!(
            "{field} must be a non-empty string"
        )));
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> Result<(), PackError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains('\0')
        || (value.len() >= 2 && value.as_bytes()[1] == b':')
        || value
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(PackError::new(format!(
            "asset path {value:?} must be a safe relative POSIX path"
        )));
    }
    Ok(())
}

fn extension(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(_, extension)| extension)
        .map(str::to_ascii_lowercase)
}

fn secure_file(root: &Path, relative: &str) -> Result<PathBuf, PackError> {
    safe_relative_path(relative)?;
    let mut candidate = root.to_path_buf();
    for component in relative.split('/') {
        candidate.push(component);
        let metadata = fs::symlink_metadata(&candidate).map_err(|error| {
            PackError::new(format!("cannot access referenced file {relative}: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(PackError::new(format!(
                "referenced path {relative} contains a symlink"
            )));
        }
    }
    let canonical = fs::canonicalize(&candidate)?;
    if !canonical.starts_with(root) || !fs::metadata(&canonical)?.is_file() {
        return Err(PackError::new(format!(
            "referenced path {relative} is not a regular file inside the pack"
        )));
    }
    Ok(canonical)
}

fn validate_png_dimensions(path: &Path, atlas: &PackAtlas) -> Result<(), PackError> {
    let mut header = [0_u8; 24];
    File::open(path)?
        .read_exact(&mut header)
        .map_err(|error| PackError::new(format!("cannot read atlas PNG header: {error}")))?;
    if header[..8] != [137, 80, 78, 71, 13, 10, 26, 10] || &header[12..16] != b"IHDR" {
        return Err(PackError::new("atlas is not a valid PNG"));
    }
    let width = u32::from_be_bytes(header[16..20].try_into().expect("four-byte width"));
    let height = u32::from_be_bytes(header[20..24].try_into().expect("four-byte height"));
    let expected_width = atlas
        .columns
        .checked_mul(atlas.frame_width)
        .ok_or_else(|| PackError::new("atlas width overflows"))?;
    let expected_height = atlas
        .rows
        .checked_mul(atlas.frame_height)
        .ok_or_else(|| PackError::new("atlas height overflows"))?;
    if width != expected_width || height != expected_height {
        return Err(PackError::new(format!(
            "atlas is {width}x{height}; expected {expected_width}x{expected_height}"
        )));
    }
    Ok(())
}

fn managed_root(path: &Path, create: bool) -> Result<PathBuf, PackError> {
    if create {
        fs::create_dir_all(path)?;
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| PackError::new(format!("cannot access managed packs root: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PackError::new(
            "managed packs root must be a regular directory, not a symlink",
        ));
    }
    Ok(fs::canonicalize(path)?)
}

fn initialize_managed_root(root: &Path) -> Result<(), PackError> {
    let marker = root.join(MANAGED_ROOT_MARKER);
    match fs::symlink_metadata(&marker) {
        Ok(_) => require_managed_root(root),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if fs::read_dir(root)?.next().transpose()?.is_some() {
                return Err(PackError::new(
                    "refusing to initialize a non-empty directory as an external packs root",
                ));
            }
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(marker)?;
            file.write_all(b"virtual-pet-external-packs-v1\n")?;
            file.sync_all()?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn require_managed_root(root: &Path) -> Result<(), PackError> {
    let marker = root.join(MANAGED_ROOT_MARKER);
    let metadata = fs::symlink_metadata(&marker)
        .map_err(|_| PackError::new("directory is not an app-managed external packs root"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PackError::new("external packs root marker is invalid"));
    }
    let mut contents = String::new();
    File::open(marker)?.read_to_string(&mut contents)?;
    if contents != "virtual-pet-external-packs-v1\n" {
        return Err(PackError::new("external packs root marker is invalid"));
    }
    Ok(())
}

fn create_temporary_directory(root: &Path, id: &str) -> Result<PathBuf, PackError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..100_u32 {
        let path = root.join(format!(
            ".install-{id}-{}-{timestamp}-{attempt}",
            process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(PackError::new(
        "could not allocate a temporary installation directory",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let name = format!(
                "virtual-pet-packs-test-{}-{}-{sequence}",
                process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let path = std::env::temp_dir().join(name);
            fs::create_dir(&path).expect("create unique test directory");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn manifest() -> Value {
        json!({
            "schemaVersion": 1,
            "id": "example.test-pet",
            "name": "Test Pet",
            "version": "1.2.3-beta.1",
            "species": "test-pet",
            "description": "A test pet.",
            "authors": [{ "name": "Test Author" }],
            "atlas": {
                "file": "assets/atlas.png",
                "columns": 2,
                "rows": 4,
                "frameWidth": 8,
                "frameHeight": 8
            },
            "animations": {
                "idle": { "row": 0, "frames": [0, 1], "frameDurationMs": 400, "loop": true },
                "walk": { "row": 1, "frames": [0, 1], "frameDurationMs": 120, "loop": true },
                "run": { "row": 2, "frames": [0, 1], "frameDurationMs": 80, "loop": true },
                "sleep": { "row": 3, "frames": [0, 1], "frameDurationMs": 500, "loop": true }
            },
            "sounds": {
                "sleep": { "file": "sounds/snore.mp3", "volume": 0.3, "loop": true }
            },
            "license": {
                "spdx": "MIT",
                "assets": [
                    {
                        "path": "assets/atlas.png",
                        "creator": "Test Author",
                        "source": "https://example.com/atlas",
                        "spdx": "MIT",
                        "modifications": "Unmodified"
                    },
                    {
                        "path": "sounds/snore.mp3",
                        "creator": "Test Author",
                        "source": "https://example.com/snore",
                        "spdx": "CC0-1.0",
                        "modifications": "Unmodified"
                    }
                ]
            }
        })
    }

    fn write_pack(root: &Path, manifest: &Value) {
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::create_dir_all(root.join("sounds")).unwrap();
        fs::write(
            root.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(manifest).unwrap(),
        )
        .unwrap();
        let mut png = vec![0_u8; 24];
        png[..8].copy_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&16_u32.to_be_bytes());
        png[20..24].copy_from_slice(&32_u32.to_be_bytes());
        fs::write(root.join("assets/atlas.png"), png).unwrap();
        fs::write(root.join("sounds/snore.mp3"), b"test audio").unwrap();
    }

    #[test]
    fn validates_installs_lists_and_removes_only_referenced_assets() {
        let temporary = TestDirectory::new();
        let source = temporary.0.join("source");
        let installed = temporary.0.join("installed");
        write_pack(&source, &manifest());
        fs::write(source.join("not-referenced.txt"), b"do not install").unwrap();

        assert_eq!(validate_pack(&source).unwrap().id, "example.test-pet");
        install_pack(&source, &installed).unwrap();
        let target = installed.join("example.test-pet");
        assert!(target.join(MANIFEST_FILE).is_file());
        assert!(target.join("assets/atlas.png").is_file());
        assert!(target.join("sounds/snore.mp3").is_file());
        assert!(!target.join("not-referenced.txt").exists());
        assert_eq!(list_installed_packs(&installed).unwrap().len(), 1);

        remove_installed_pack(&installed, "example.test-pet").unwrap();
        assert!(!target.exists());
        assert!(installed.is_dir());
    }

    #[test]
    fn rejects_traversal_invalid_frames_and_missing_license() {
        let temporary = TestDirectory::new();
        for (index, mutation) in [
            ("/atlas/file", json!("../outside.png")),
            ("/animations/walk/frames", json!([2])),
            ("/license/assets", json!([])),
        ]
        .into_iter()
        .enumerate()
        {
            let source = temporary.0.join(format!("source-{index}"));
            let mut value = manifest();
            *value.pointer_mut(mutation.0).unwrap() = mutation.1;
            write_pack(&source, &value);
            assert!(validate_pack(&source).is_err());
        }
    }

    #[test]
    fn rejects_bad_atlas_dimensions_and_sound_properties() {
        let temporary = TestDirectory::new();
        let bad_dimensions = temporary.0.join("dimensions");
        write_pack(&bad_dimensions, &manifest());
        let mut png = fs::read(bad_dimensions.join("assets/atlas.png")).unwrap();
        png[16..20].copy_from_slice(&15_u32.to_be_bytes());
        fs::write(bad_dimensions.join("assets/atlas.png"), png).unwrap();
        assert!(validate_pack(&bad_dimensions).is_err());

        let bad_sound = temporary.0.join("sound");
        let mut value = manifest();
        value["sounds"]["sleep"]["volume"] = json!(1.1);
        write_pack(&bad_sound, &value);
        assert!(validate_pack(&bad_sound).is_err());
    }

    #[test]
    fn removal_rejects_traversal_and_preserves_siblings() {
        let temporary = TestDirectory::new();
        let installed = temporary.0.join("installed");
        let sibling = temporary.0.join("keep");
        fs::create_dir(&installed).unwrap();
        fs::create_dir(&sibling).unwrap();

        assert!(remove_installed_pack(&installed, "../keep").is_err());
        assert!(sibling.is_dir());
        assert!(installed.is_dir());
    }

    #[test]
    fn never_removes_a_pack_from_an_unmanaged_or_bundled_root() {
        let temporary = TestDirectory::new();
        let bundled = temporary.0.join("bundled");
        let puppy = bundled.join("virtual-pet-desktop.puppy");
        fs::create_dir_all(&puppy).unwrap();

        assert!(remove_installed_pack(&bundled, "virtual-pet-desktop.puppy").is_err());
        assert!(puppy.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_assets_and_installed_entries() {
        use std::os::unix::fs::symlink;

        let temporary = TestDirectory::new();
        let source = temporary.0.join("source");
        write_pack(&source, &manifest());
        let outside = temporary.0.join("outside.png");
        fs::write(&outside, fs::read(source.join("assets/atlas.png")).unwrap()).unwrap();
        fs::remove_file(source.join("assets/atlas.png")).unwrap();
        symlink(&outside, source.join("assets/atlas.png")).unwrap();
        assert!(validate_pack(&source).is_err());

        let installed = temporary.0.join("installed");
        fs::create_dir(&installed).unwrap();
        symlink(&source, installed.join("example.test-pet")).unwrap();
        assert!(list_installed_packs(&installed).is_err());
        assert!(remove_installed_pack(&installed, "example.test-pet").is_err());
    }
}
