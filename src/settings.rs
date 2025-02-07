use std::path::{Path, PathBuf};

use iced::{
    widget::{column, container, pick_list, row, text},
    Alignment, Element,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettingsTheme {
    Light,
    Dark,
}

impl std::fmt::Display for SettingsTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SettingsTheme {
    pub const ALL: [Self; 2] = [SettingsTheme::Light, SettingsTheme::Dark];
}

impl Default for SettingsTheme {
    fn default() -> Self {
        let system_use_dark = iced::Theme::default() == iced::Theme::Dark;
        if system_use_dark {
            SettingsTheme::Dark
        } else {
            SettingsTheme::Light
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub theme: SettingsTheme,
}

#[derive(Clone, Debug)]
pub enum MessageSettings {
    ThemeSelected(SettingsTheme),
}

const CONFIG_FILE_NAME: &str = "config.json";

pub fn read_settings(path: &Path) -> Option<Settings> {
    let path = path.to_path_buf().join(CONFIG_FILE_NAME);

    let Ok(file) = std::fs::File::open(&path) else {
        return None;
    };

    let Ok(v) = serde_json::from_reader(&file) else {
        return None;
    };

    v
}

pub fn serialize_settings(settings: &Settings) -> String {
    serde_json::to_string_pretty(settings).unwrap()
}

pub async fn write_config(path: PathBuf, settings: String) -> std::io::Result<()> {
    let path = path.join(CONFIG_FILE_NAME);
    let tmp_path = path.clone().with_extension(".json.tmp");

    let mut file = tokio::fs::File::create(&tmp_path).await?;
    file.write_all(settings.as_bytes()).await?;
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

impl Settings {
    pub fn update(&mut self, message: MessageSettings) {
        match message {
            MessageSettings::ThemeSelected(settings_theme) => {
                self.theme = settings_theme;
            }
        }
    }

    pub fn view<'a>(&self) -> Element<'a, MessageSettings> {
        let labelled_row = |s| row![].push(container(text(s)).width(120.0));
        column![]
            .push(labelled_row("Theme").push(pick_list(
                SettingsTheme::ALL,
                Some(self.theme),
                MessageSettings::ThemeSelected,
            )))
            .align_x(Alignment::Start)
            .into()
    }
}
