use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub const BUILTIN_AUDIO_ID: &str = "builtin:beautiful-adhan";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub location_id: String,
    pub calculation_method: String,
    pub asr_method: String,
    pub high_latitude_rule: String,
    pub master_enabled: bool,
    pub fajr_enabled: bool,
    pub dhuhr_enabled: bool,
    pub asr_enabled: bool,
    pub maghrib_enabled: bool,
    pub isha_enabled: bool,
    pub volume: f32,
    pub regular_audio: String,
    pub fajr_audio: String,
    pub start_on_login: bool,
    pub start_hidden: bool,
    pub muted_until_unix: i64,
    pub last_played_prayer: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            location_id: "eg-cairo".into(),
            calculation_method: "Egyptian".into(),
            asr_method: "Standard".into(),
            high_latitude_rule: "Middle of the night".into(),
            master_enabled: true,
            fajr_enabled: true,
            dhuhr_enabled: true,
            asr_enabled: true,
            maghrib_enabled: true,
            isha_enabled: true,
            volume: 0.78,
            regular_audio: BUILTIN_AUDIO_ID.into(),
            // The bundled recording is a regular Azan. Until a Fajr-specific
            // recording is chosen, the regular recording is used as fallback.
            fajr_audio: String::new(),
            start_on_login: true,
            start_hidden: true,
            muted_until_unix: 0,
            last_played_prayer: String::new(),
        }
    }
}

impl Settings {
    pub fn prayer_enabled(&self, prayer: crate::prayer::PrayerKind) -> bool {
        use crate::prayer::PrayerKind;
        self.master_enabled
            && match prayer {
                PrayerKind::Fajr => self.fajr_enabled,
                PrayerKind::Sunrise => false,
                PrayerKind::Dhuhr => self.dhuhr_enabled,
                PrayerKind::Asr => self.asr_enabled,
                PrayerKind::Maghrib => self.maghrib_enabled,
                PrayerKind::Isha => self.isha_enabled,
            }
    }

    pub fn set_prayer_enabled(&mut self, prayer: crate::prayer::PrayerKind, enabled: bool) {
        use crate::prayer::PrayerKind;
        match prayer {
            PrayerKind::Fajr => self.fajr_enabled = enabled,
            PrayerKind::Sunrise => {}
            PrayerKind::Dhuhr => self.dhuhr_enabled = enabled,
            PrayerKind::Asr => self.asr_enabled = enabled,
            PrayerKind::Maghrib => self.maghrib_enabled = enabled,
            PrayerKind::Isha => self.isha_enabled = enabled,
        }
    }

    pub fn audio_for(&self, prayer: crate::prayer::PrayerKind) -> &str {
        if prayer == crate::prayer::PrayerKind::Fajr && !self.fajr_audio.is_empty() {
            &self.fajr_audio
        } else {
            &self.regular_audio
        }
    }
}

#[derive(Clone, Debug)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new() -> Result<Self> {
        let dirs = ProjectDirs::from("org", "azanboki", "AzanBoki")
            .context("the operating system did not provide a configuration directory")?;
        fs::create_dir_all(dirs.config_dir()).context("failed to create the settings directory")?;
        Ok(Self {
            path: dirs.config_dir().join("settings.json"),
        })
    }

    pub fn data_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("org", "azanboki", "AzanBoki")
            .context("the operating system did not provide an application data directory")?;
        fs::create_dir_all(dirs.data_local_dir())
            .context("failed to create the application data directory")?;
        Ok(dirs.data_local_dir().to_path_buf())
    }

    pub fn load(&self) -> Result<Settings> {
        if !self.path.exists() {
            let settings = Settings::default();
            self.save(&settings)?;
            return Ok(settings);
        }

        let raw = fs::read_to_string(&self.path).context("failed to read settings")?;
        let mut settings: Settings =
            serde_json::from_str(&raw).context("failed to parse settings")?;
        settings.volume = settings.volume.clamp(0.0, 1.0);
        Ok(settings)
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(&self.path, json).context("failed to save settings")
    }

    #[cfg(test)]
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let path = std::env::temp_dir().join(format!(
            "azanboki-settings-{}-{}.json",
            std::process::id(),
            jiff::Timestamp::now().as_nanosecond()
        ));
        let store = SettingsStore::at(path.clone());
        let settings = Settings {
            location_id: "gb-london".into(),
            volume: 0.42,
            ..Settings::default()
        };
        store.save(&settings).unwrap();
        assert_eq!(store.load().unwrap(), settings);
        let _ = fs::remove_file(path);
    }
}
