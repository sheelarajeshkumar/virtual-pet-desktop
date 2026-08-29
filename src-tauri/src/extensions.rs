use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};

const MARKER: &str = ".virtual-pet-behaviors-v1";
const MAX_EXTENSION_BYTES: u64 = 64 * 1024;
const MAX_STEPS: usize = 16;
const MAX_TOTAL_MS: u64 = 60_000;

#[derive(Debug)]
pub struct ExtensionError(String);

impl fmt::Display for ExtensionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionAction {
    Auto,
    Follow,
    Play,
    Bark,
    Sleep,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionStep {
    pub action: ExtensionAction,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorExtension {
    pub schema_version: u8,
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<ExtensionStep>,
}

pub fn read_extension(path: &Path) -> Result<BehaviorExtension, ExtensionError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ExtensionError(format!("cannot access behavior: {error}")))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_EXTENSION_BYTES
    {
        return Err(ExtensionError(
            "behavior must be a regular JSON file no larger than 64 KiB".to_string(),
        ));
    }
    let extension: BehaviorExtension = serde_json::from_slice(
        &fs::read(path)
            .map_err(|error| ExtensionError(format!("cannot read behavior: {error}")))?,
    )
    .map_err(|error| ExtensionError(format!("invalid behavior JSON: {error}")))?;
    validate(&extension)?;
    Ok(extension)
}

pub fn install_extension(
    source: &Path,
    managed_root: &Path,
) -> Result<BehaviorExtension, ExtensionError> {
    let extension = read_extension(source)?;
    initialize_root(managed_root)?;
    let destination = managed_root.join(format!("{}.json", extension.id));
    if destination.exists() {
        return Err(ExtensionError(format!(
            "behavior {} is already installed",
            extension.id
        )));
    }
    let json =
        serde_json::to_vec_pretty(&extension).map_err(|error| ExtensionError(error.to_string()))?;
    fs::write(destination, json)
        .map_err(|error| ExtensionError(format!("cannot install behavior: {error}")))?;
    Ok(extension)
}

pub fn list_extensions(managed_root: &Path) -> Result<Vec<BehaviorExtension>, ExtensionError> {
    if !managed_root.exists() {
        return Ok(Vec::new());
    }
    require_root(managed_root)?;
    let mut extensions = Vec::new();
    for entry in fs::read_dir(managed_root)
        .map_err(|error| ExtensionError(format!("cannot list behaviors: {error}")))?
    {
        let entry = entry.map_err(|error| ExtensionError(error.to_string()))?;
        if entry.file_name() == MARKER
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let extension = read_extension(&entry.path())?;
        let expected_name = format!("{}.json", extension.id);
        if entry.file_name().to_str() != Some(expected_name.as_str()) {
            return Err(ExtensionError(
                "behavior filename does not match its id".to_string(),
            ));
        }
        extensions.push(extension);
    }
    extensions.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(extensions)
}

pub fn remove_extension(managed_root: &Path, id: &str) -> Result<(), ExtensionError> {
    validate_id(id)?;
    require_root(managed_root)?;
    fs::remove_file(managed_root.join(format!("{id}.json")))
        .map_err(|error| ExtensionError(format!("cannot remove behavior: {error}")))
}

fn validate(extension: &BehaviorExtension) -> Result<(), ExtensionError> {
    if extension.schema_version != 1 {
        return Err(ExtensionError("schemaVersion must be 1".to_string()));
    }
    validate_id(&extension.id)?;
    if extension.name.trim().is_empty()
        || extension.name.chars().count() > 40
        || extension.description.chars().count() > 240
        || extension.name.chars().any(char::is_control)
        || extension.description.chars().any(char::is_control)
    {
        return Err(ExtensionError("behavior text is invalid".to_string()));
    }
    if extension.steps.is_empty() || extension.steps.len() > MAX_STEPS {
        return Err(ExtensionError(
            "behavior must contain 1 to 16 steps".to_string(),
        ));
    }
    let total = extension.steps.iter().try_fold(0_u64, |total, step| {
        if !(250..=10_000).contains(&step.duration_ms) {
            return Err(ExtensionError(
                "each behavior step must last 250 to 10000 ms".to_string(),
            ));
        }
        total
            .checked_add(step.duration_ms)
            .ok_or_else(|| ExtensionError("behavior duration overflows".to_string()))
    })?;
    if total > MAX_TOTAL_MS {
        return Err(ExtensionError(
            "behavior must finish within 60 seconds".to_string(),
        ));
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), ExtensionError> {
    if id.is_empty()
        || id.len() > 64
        || id.starts_with(['-', '.'])
        || id.ends_with(['-', '.'])
        || id.contains("..")
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-.".contains(&byte))
    {
        return Err(ExtensionError("behavior id is invalid".to_string()));
    }
    Ok(())
}

fn initialize_root(root: &Path) -> Result<(), ExtensionError> {
    fs::create_dir_all(root).map_err(|error| ExtensionError(error.to_string()))?;
    let marker = root.join(MARKER);
    if marker.exists() {
        return require_root(root);
    }
    if fs::read_dir(root)
        .map_err(|error| ExtensionError(error.to_string()))?
        .next()
        .is_some()
    {
        return Err(ExtensionError(
            "refusing to manage a non-empty behavior directory".to_string(),
        ));
    }
    fs::write(marker, b"virtual-pet-behaviors-v1\n")
        .map_err(|error| ExtensionError(error.to_string()))
}

fn require_root(root: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| ExtensionError(format!("cannot access behavior directory: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError(
            "behavior directory must be a regular directory".to_string(),
        ));
    }
    if fs::read_to_string(root.join(MARKER)).ok().as_deref() != Some("virtual-pet-behaviors-v1\n") {
        return Err(ExtensionError(
            "directory is not an app-managed behavior root".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn directory() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "virtual-pet-extensions-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn sample() -> BehaviorExtension {
        BehaviorExtension {
            schema_version: 1,
            id: "example.happy-dance".to_string(),
            name: "Happy Dance".to_string(),
            description: "A short allowlisted routine.".to_string(),
            steps: vec![
                ExtensionStep {
                    action: ExtensionAction::Play,
                    duration_ms: 1_000,
                },
                ExtensionStep {
                    action: ExtensionAction::Bark,
                    duration_ms: 500,
                },
            ],
        }
    }

    #[test]
    fn installs_lists_and_removes_declarative_behavior() {
        let temporary = directory();
        let source = temporary.join("source.json");
        let managed = temporary.join("managed");
        fs::write(&source, serde_json::to_vec(&sample()).unwrap()).unwrap();
        install_extension(&source, &managed).unwrap();
        assert_eq!(list_extensions(&managed).unwrap().len(), 1);
        remove_extension(&managed, "example.happy-dance").unwrap();
        assert!(list_extensions(&managed).unwrap().is_empty());
        let _ = fs::remove_dir_all(temporary);
    }

    #[test]
    fn rejects_code_like_and_unbounded_behaviors() {
        let mut extension = sample();
        extension.id = "../bad".to_string();
        assert!(validate(&extension).is_err());
        extension = sample();
        extension.steps[0].duration_ms = 60_001;
        assert!(validate(&extension).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_behavior_files() {
        use std::os::unix::fs::symlink;

        let temporary = directory();
        let target = temporary.join("target.json");
        let link = temporary.join("link.json");
        fs::write(&target, serde_json::to_vec(&sample()).unwrap()).unwrap();
        symlink(&target, &link).unwrap();
        assert!(read_extension(&link).is_err());
        let _ = fs::remove_dir_all(temporary);
    }
}
