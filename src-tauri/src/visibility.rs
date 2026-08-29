use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;

/// The default global shortcut used to hide or show the pet window.
pub const DEFAULT_VISIBILITY_SHORTCUT: &str = "CommandOrControl+Shift+P";

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
}

impl Visibility {
    pub const fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }

    const fn toggled(self) -> Self {
        match self {
            Self::Visible => Self::Hidden,
            Self::Hidden => Self::Visible,
        }
    }
}

/// Persistable visibility state. Tauri integration owns applying this state to
/// the actual window and registering `shortcut` with the global-shortcut plugin.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityState {
    pub visibility: Visibility,
    pub shortcut: VisibilityShortcut,
}

impl Default for VisibilityState {
    fn default() -> Self {
        Self {
            visibility: Visibility::Visible,
            shortcut: VisibilityShortcut::default(),
        }
    }
}

impl VisibilityState {
    pub fn toggle(&mut self) -> Visibility {
        self.visibility = self.visibility.toggled();
        self.visibility
    }

    pub fn set_shortcut(&mut self, value: &str) -> Result<(), ShortcutError> {
        self.shortcut = value.parse()?;
        Ok(())
    }
}

/// A normalized Tauri accelerator string. Parsing deliberately requires a
/// modifier, so normal typing cannot hide the pet.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct VisibilityShortcut(String);

impl Default for VisibilityShortcut {
    fn default() -> Self {
        DEFAULT_VISIBILITY_SHORTCUT
            .parse()
            .expect("default visibility shortcut is valid")
    }
}

impl VisibilityShortcut {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VisibilityShortcut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortcutError(&'static str);

impl fmt::Display for ShortcutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for ShortcutError {}

impl std::str::FromStr for VisibilityShortcut {
    type Err = ShortcutError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() || value.len() > 64 {
            return Err(ShortcutError(
                "shortcut must be between 1 and 64 characters",
            ));
        }

        let mut command_or_control = false;
        let mut command = false;
        let mut control = false;
        let mut alt = false;
        let mut shift = false;
        let mut super_key = false;
        let mut key = None;

        for part in value.split('+') {
            let part = part.trim();
            if part.is_empty() {
                return Err(ShortcutError("shortcut contains an empty key part"));
            }
            match part.to_ascii_lowercase().as_str() {
                "commandorcontrol" | "cmdorctrl" => {
                    command_or_control = set_once(
                        command_or_control,
                        "shortcut contains CommandOrControl more than once",
                    )?
                }
                "command" | "cmd" => {
                    command = set_once(command, "shortcut contains Command more than once")?
                }
                "control" | "ctrl" => {
                    control = set_once(control, "shortcut contains Control more than once")?
                }
                "alt" | "option" => alt = set_once(alt, "shortcut contains Alt more than once")?,
                "shift" => shift = set_once(shift, "shortcut contains Shift more than once")?,
                "super" | "meta" | "win" => {
                    super_key = set_once(super_key, "shortcut contains Super more than once")?
                }
                _ if valid_key(part) && key.replace(canonical_key(part)).is_none() => {}
                _ if valid_key(part) => {
                    return Err(ShortcutError("shortcut contains more than one key"))
                }
                _ => return Err(ShortcutError("shortcut contains an unsupported key")),
            }
        }

        if key.is_none() {
            return Err(ShortcutError("shortcut needs a non-modifier key"));
        }
        if !(command_or_control || command || control || alt || shift || super_key) {
            return Err(ShortcutError("shortcut needs at least one modifier"));
        }
        if command_or_control && (command || control) {
            return Err(ShortcutError(
                "CommandOrControl cannot be combined with Command or Control",
            ));
        }

        let mut parts = Vec::with_capacity(7);
        if command_or_control {
            parts.push("CommandOrControl");
        }
        if command {
            parts.push("Command");
        }
        if control {
            parts.push("Control");
        }
        if alt {
            parts.push("Alt");
        }
        if shift {
            parts.push("Shift");
        }
        if super_key {
            parts.push("Super");
        }
        let mut shortcut = parts.join("+");
        shortcut.push('+');
        shortcut.push_str(key.as_deref().expect("key checked above"));
        Ok(Self(shortcut))
    }
}

impl<'de> Deserialize<'de> for VisibilityShortcut {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

fn set_once(current: bool, message: &'static str) -> Result<bool, ShortcutError> {
    if current {
        Err(ShortcutError(message))
    } else {
        Ok(true)
    }
}

fn valid_key(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "SPACE"
            | "TAB"
            | "ENTER"
            | "ESCAPE"
            | "BACKSPACE"
            | "DELETE"
            | "INSERT"
            | "HOME"
            | "END"
            | "PAGEUP"
            | "PAGEDOWN"
            | "ARROWUP"
            | "ARROWDOWN"
            | "ARROWLEFT"
            | "ARROWRIGHT"
    ) || (value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric())
        || upper
            .strip_prefix('F')
            .and_then(|number| number.parse::<u8>().ok())
            .is_some_and(|number| (1..=24).contains(&number))
}

fn canonical_key(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "space" => "Space".to_string(),
        "tab" => "Tab".to_string(),
        "enter" => "Enter".to_string(),
        "escape" => "Escape".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" => "Delete".to_string(),
        "insert" => "Insert".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" => "PageUp".to_string(),
        "pagedown" => "PageDown".to_string(),
        "arrowup" => "ArrowUp".to_string(),
        "arrowdown" => "ArrowDown".to_string(),
        "arrowleft" => "ArrowLeft".to_string(),
        "arrowright" => "ArrowRight".to_string(),
        _ => value.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_is_normalized_and_aliases_are_accepted() {
        let shortcut: VisibilityShortcut = "ctrl + shift + p".parse().unwrap();
        assert_eq!(shortcut.as_str(), "Control+Shift+P");
    }

    #[test]
    fn shortcut_rejects_unsafe_or_ambiguous_inputs() {
        for invalid in ["P", "Shift", "Ctrl++P", "Ctrl+P+Q", "CmdOrCtrl+Ctrl+P"] {
            assert!(invalid.parse::<VisibilityShortcut>().is_err(), "{invalid}");
        }
    }

    #[test]
    fn visibility_actions_are_idempotent_and_toggleable() {
        let mut state = VisibilityState::default();
        assert_eq!(state.toggle(), Visibility::Hidden);
        assert_eq!(state.toggle(), Visibility::Visible);
    }

    #[test]
    fn persisted_shortcut_is_validated_on_read() {
        let state: VisibilityState =
            serde_json::from_str(r#"{"visibility":"hidden","shortcut":"cmdorctrl+shift+p"}"#)
                .unwrap();
        assert_eq!(state.visibility, Visibility::Hidden);
        assert_eq!(state.shortcut.as_str(), DEFAULT_VISIBILITY_SHORTCUT);
        assert!(serde_json::from_str::<VisibilityState>(
            r#"{"visibility":"visible","shortcut":"P"}"#
        )
        .is_err());
    }
}
