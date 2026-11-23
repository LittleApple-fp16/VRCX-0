#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use vrcx_0_runtime_host_desktop::tray_shortcut::{TrayShortcutBinding, TrayShortcutError};

use crate::{error::AppError, state::AppState};

#[cfg(windows)]
mod native;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum HighRiskRoom {
    Public,
    GroupPublic,
    GroupPlus,
    FriendsPlus,
    Friends,
    InvitePlus,
    Invite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum HighRiskNickname {
    English,
    Suffix,
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum HighRiskRank {
    Visitor,
    NewUser,
    User,
    KnownUser,
    TrustedUser,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum HighRiskMark {
    ProbableTroll,
    Troll,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(default, rename_all = "camelCase")]
pub struct HighRiskDetectionSettings {
    pub rooms: Vec<HighRiskRoom>,
    pub nickname: HighRiskNickname,
    pub ranks: Vec<HighRiskRank>,
    pub marks: Vec<HighRiskMark>,
    pub auto_block: bool,
    pub shortcut: TrayShortcutBinding,
}

impl Default for HighRiskDetectionSettings {
    fn default() -> Self {
        Self {
            rooms: vec![
                HighRiskRoom::Public,
                HighRiskRoom::GroupPublic,
                HighRiskRoom::GroupPlus,
            ],
            nickname: HighRiskNickname::English,
            ranks: vec![HighRiskRank::NewUser, HighRiskRank::Visitor],
            marks: Vec::new(),
            auto_block: true,
            shortcut: TrayShortcutBinding {
                control: true,
                alt: false,
                shift: false,
                key: "KeyF".into(),
            },
        }
    }
}

pub(super) fn setup(app: &AppHandle, state: &AppState) {
    #[cfg(windows)]
    native::setup(app, state);
    #[cfg(not(windows))]
    let _ = (app, state);
}

#[tauri::command]
#[specta::specta]
pub fn app__get_high_risk_detection(app_handle: AppHandle) -> Option<HighRiskDetectionSettings> {
    #[cfg(windows)]
    return native::settings(&app_handle);
    #[cfg(not(windows))]
    {
        let _ = app_handle;
        None
    }
}

#[tauri::command]
#[specta::specta]
pub async fn app__set_high_risk_detection(
    app_handle: AppHandle,
    settings: HighRiskDetectionSettings,
) -> Result<Option<TrayShortcutError>, AppError> {
    #[cfg(windows)]
    return super::tray_shortcut::on_main_thread(&app_handle, move |app| {
        native::save(app, settings)
    })
    .await?;
    #[cfg(not(windows))]
    {
        let _ = (app_handle, settings);
        Ok(Some(TrayShortcutError::Unsupported))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn app__check_high_risk_shortcut(
    app_handle: AppHandle,
    binding: TrayShortcutBinding,
) -> Result<Option<TrayShortcutError>, AppError> {
    #[cfg(windows)]
    return super::tray_shortcut::on_main_thread(&app_handle, move |app| {
        native::check_shortcut(app, &binding).err()
    })
    .await;
    #[cfg(not(windows))]
    {
        let _ = (app_handle, binding);
        Ok(Some(TrayShortcutError::Unsupported))
    }
}
