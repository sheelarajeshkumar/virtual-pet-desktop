use std::{fmt, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{care::CareState, engine::PetSettings};

const MAX_BACKUP_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct BackupError(String);

impl fmt::Display for BackupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub schema_version: u8,
    pub settings: PetSettings,
    pub care: CareState,
}

impl BackupFile {
    pub const fn new(settings: PetSettings, care: CareState) -> Self {
        Self {
            schema_version: 1,
            settings,
            care,
        }
    }
}

pub fn write_backup(path: &Path, backup: &BackupFile) -> Result<(), BackupError> {
    if fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_symlink() || metadata.is_dir())
    {
        return Err(BackupError(
            "backup destination cannot be a symlink or directory".to_string(),
        ));
    }
    let json = serde_json::to_vec_pretty(backup).map_err(|error| BackupError(error.to_string()))?;
    fs::write(path, json).map_err(|error| BackupError(format!("cannot write backup: {error}")))
}

pub fn read_backup(path: &Path) -> Result<BackupFile, BackupError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| BackupError(format!("cannot access backup: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_BACKUP_BYTES
    {
        return Err(BackupError(
            "backup must be a regular JSON file no larger than 1 MiB".to_string(),
        ));
    }
    let backup: BackupFile = serde_json::from_slice(
        &fs::read(path).map_err(|error| BackupError(format!("cannot read backup: {error}")))?,
    )
    .map_err(|error| BackupError(format!("invalid backup JSON: {error}")))?;
    if backup.schema_version != 1 {
        return Err(BackupError("unsupported backup schema version".to_string()));
    }
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "virtual-pet-{name}-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn round_trips_settings_and_care() {
        let path = path("backup");
        let settings = PetSettings {
            name: "Backup Pet".to_string(),
            ..PetSettings::default()
        };
        let mut care = CareState::default();
        for now in 1..=3 {
            care.apply(
                crate::care::CareAction::Feed,
                now,
                crate::care::CareDifficulty::Normal,
                true,
            );
        }
        write_backup(&path, &BackupFile::new(settings, care)).unwrap();
        let restored = read_backup(&path).unwrap();
        assert_eq!(restored.settings.name, "Backup Pet");
        assert_eq!(restored.care.food, 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_unknown_schema() {
        let path = path("bad-backup");
        fs::write(&path, r#"{"schemaVersion":2,"settings":{},"care":{}}"#).unwrap();
        assert!(read_backup(&path).is_err());
        let _ = fs::remove_file(path);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_backup_files() {
        use std::os::unix::fs::symlink;

        let target = path("backup-target");
        let link = path("backup-link");
        write_backup(
            &target,
            &BackupFile::new(PetSettings::default(), CareState::default()),
        )
        .unwrap();
        symlink(&target, &link).unwrap();
        assert!(read_backup(&link).is_err());
        assert!(write_backup(
            &link,
            &BackupFile::new(PetSettings::default(), CareState::default())
        )
        .is_err());
        let _ = fs::remove_file(link);
        let _ = fs::remove_file(target);
    }
}
