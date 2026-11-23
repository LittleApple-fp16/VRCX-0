use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use serde::Deserialize;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_winrt_notification::{Sound, Toast};
use vrcx_0_application::social::ModerationSyncMutationInput;
use vrcx_0_application_core::RuntimeAuthScopeSnapshot;
use vrcx_0_application_game::PlayerListSnapshotPlayer;
use vrcx_0_core::{
    location::parse_location,
    trust::{compute_trust_level, TrustRank},
    OwnerId,
};
use vrcx_0_runtime_host_desktop::local_data::ConfigWriteEntry;

use super::*;
use crate::bootstrap::{
    adapters::windows_notification_app_id, shared::app_language, tray_shortcut,
};

const CONFIG_KEY: &str = "highRiskDetection";
const ROBOT_AVATAR_ID: &str = "avtr_c38a1615-5bf5-42b4-84eb-a8b6c37cbd11";
const SCAN_INTERVAL: Duration = Duration::from_millis(500);
const LOOKUP_RETRY: Duration = Duration::from_secs(15);
const TOAST_FALLBACK_INTERVAL: Duration = Duration::from_secs(30);
const MAX_LOOKUPS: usize = 4;

#[derive(Clone)]
struct PendingPlayer {
    display_name: String,
    blocked: bool,
}

struct DetectionState {
    settings: HighRiskDetectionSettings,
    scope: Option<RuntimeAuthScopeSnapshot>,
    pending: BTreeMap<String, PendingPlayer>,
    processing: bool,
    active_shortcut: Option<Shortcut>,
    toast_id: u64,
    next_alert: Option<Instant>,
}

struct Shared(Mutex<DetectionState>);

impl Shared {
    fn lock(&self) -> MutexGuard<'_, DetectionState> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }
}

impl DetectionState {
    fn refresh_scope(&mut self, scope: &RuntimeAuthScopeSnapshot) {
        if !same_scope(self.scope.as_ref(), scope) {
            self.scope = Some(scope.clone());
            self.pending.clear();
            self.processing = false;
            self.next_alert = None;
            self.toast_id += 1;
        }
    }
}

fn same_scope(
    current: Option<&RuntimeAuthScopeSnapshot>,
    scope: &RuntimeAuthScopeSnapshot,
) -> bool {
    current.is_some_and(|current| {
        current.active == scope.active
            && current.generation == scope.generation
            && current.current_user_id == scope.current_user_id
            && current.endpoint == scope.endpoint
    })
}

pub(super) fn setup(app: &AppHandle, state: &AppState) {
    let raw = state.runtime_host().config_string(CONFIG_KEY, "");
    let mut settings: HighRiskDetectionSettings = serde_json::from_str(&raw).unwrap_or_default();
    if settings.shortcut.validate().is_err() {
        settings.shortcut = HighRiskDetectionSettings::default().shortcut;
    }
    app.manage(Shared(Mutex::new(DetectionState {
        settings,
        scope: None,
        pending: BTreeMap::new(),
        processing: false,
        active_shortcut: None,
        toast_id: 0,
        next_alert: None,
    })));
    tauri::async_runtime::spawn(run(app.clone()));
}

pub(super) fn settings(app: &AppHandle) -> Option<HighRiskDetectionSettings> {
    Some(app.try_state::<Shared>()?.lock().settings.clone())
}

pub(super) fn check_shortcut(
    app: &AppHandle,
    binding: &TrayShortcutBinding,
) -> Result<(), TrayShortcutError> {
    let shared = app
        .try_state::<Shared>()
        .ok_or(TrayShortcutError::Unavailable)?;
    if app
        .try_state::<tauri_plugin_global_shortcut::GlobalShortcut<tauri::Wry>>()
        .is_none()
    {
        return Err(TrayShortcutError::Unavailable);
    }
    let shortcut = binding
        .accelerator()
        .parse::<Shortcut>()
        .map_err(|_| TrayShortcutError::Invalid)?;
    if shared.lock().active_shortcut.as_ref() == Some(&shortcut) {
        return Ok(());
    }
    tray_shortcut::probe(binding)
}

pub(super) fn save(
    app: &AppHandle,
    settings: HighRiskDetectionSettings,
) -> Result<Option<TrayShortcutError>, AppError> {
    let shared = app.state::<Shared>();
    if shared.lock().settings.shortcut != settings.shortcut {
        if let Err(error) = check_shortcut(app, &settings.shortcut) {
            return Ok(Some(error));
        }
    }
    app.state::<AppState>()
        .runtime_host()
        .local_data()
        .config_set_values(vec![ConfigWriteEntry {
            key: CONFIG_KEY.into(),
            value: serde_json::to_string(&settings)?,
        }])?;
    shared.lock().settings = settings;
    sync_shortcut(app, true);
    Ok(None)
}

// Register only while an event is pending so Ctrl+F remains available for search.
fn sync_shortcut(app: &AppHandle, register: bool) -> bool {
    let shared = app.state::<Shared>();
    let mut state = shared.lock();
    let wanted = if state.pending.is_empty() {
        None
    } else {
        state
            .settings
            .shortcut
            .accelerator()
            .parse::<Shortcut>()
            .ok()
    };
    if wanted == state.active_shortcut {
        return wanted.is_some();
    }
    if let Some(active) = state.active_shortcut.take() {
        if let Err(error) = app.global_shortcut().unregister(active) {
            tracing::warn!(%error, "Failed to release the high-risk handling shortcut");
            state.active_shortcut = Some(active);
            return false;
        }
    }
    let Some(shortcut) = wanted.filter(|_| register) else {
        return false;
    };
    if app
        .try_state::<tauri_plugin_global_shortcut::GlobalShortcut<tauri::Wry>>()
        .is_none()
    {
        return false;
    }
    let result = app
        .global_shortcut()
        .on_shortcut(shortcut, |app, shortcut, event| {
            if event.state() != ShortcutState::Pressed || tray_shortcut::is_recording(app) {
                return;
            }
            let app = app.clone();
            let shortcut = *shortcut;
            tauri::async_runtime::spawn(async move { process_pending(&app, shortcut).await });
        });
    match result {
        Ok(()) => {
            state.active_shortcut = Some(shortcut);
            true
        }
        Err(error) => {
            tracing::warn!(%error, "Failed to register the high-risk handling shortcut");
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Encounter {
    user_id: String,
    joined_at_ms: i64,
}

impl From<&PlayerListSnapshotPlayer> for Encounter {
    fn from(player: &PlayerListSnapshotPlayer) -> Self {
        Self {
            user_id: player.user_id.clone(),
            joined_at_ms: player.joined_at_ms,
        }
    }
}

#[derive(PartialEq, Eq)]
struct RoomVisit {
    location: String,
    started_at: String,
}

fn room_kind(location: &str) -> Option<HighRiskRoom> {
    let parsed = parse_location(location);
    if !parsed.is_real_instance || parsed.instance_id.is_empty() {
        return None;
    }
    Some(match parsed.access_type_name.as_str() {
        "public" => HighRiskRoom::Public,
        "groupPublic" => HighRiskRoom::GroupPublic,
        "groupPlus" => HighRiskRoom::GroupPlus,
        "friends+" => HighRiskRoom::FriendsPlus,
        "friends" => HighRiskRoom::Friends,
        "invite+" => HighRiskRoom::InvitePlus,
        "invite" => HighRiskRoom::Invite,
        _ => return None,
    })
}

fn current_room(
    state: &AppState,
    settings: &HighRiskDetectionSettings,
) -> (Option<RoomVisit>, Vec<PlayerListSnapshotPlayer>) {
    if settings.rooms.is_empty() || (settings.ranks.is_empty() && settings.marks.is_empty()) {
        return (None, Vec::new());
    }
    let snapshot = state.runtime_host().current_player_list_snapshot("");
    let context = snapshot.context;
    if context.player_facts_known != Some(true)
        || !room_kind(&context.location).is_some_and(|kind| settings.rooms.contains(&kind))
    {
        return (None, Vec::new());
    }
    (
        Some(RoomVisit {
            location: context.location,
            started_at: context.created_at,
        }),
        snapshot.players,
    )
}

fn nickname_matches(mode: HighRiskNickname, name: &str) -> bool {
    let name = name.trim();
    match mode {
        HighRiskNickname::Ignore => true,
        HighRiskNickname::English => {
            name.bytes().any(|byte| byte.is_ascii_alphabetic())
                && name.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric()
                        || matches!(byte, b' ' | b'_' | b'-' | b'.' | b'\'')
                })
        }
        HighRiskNickname::Suffix => name.rsplit_once(' ').is_some_and(|(prefix, suffix)| {
            !prefix.trim().is_empty()
                && suffix.len() == 4
                && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
        }),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    id: String,
    display_name: String,
    tags: Vec<String>,
}

fn profile_matches(
    settings: &HighRiskDetectionSettings,
    display_name: &str,
    tags: &[String],
) -> bool {
    let trust = compute_trust_level(tags, "");
    let rank = match trust.rank {
        TrustRank::Visitor => HighRiskRank::Visitor,
        TrustRank::NewUser => HighRiskRank::NewUser,
        TrustRank::User => HighRiskRank::User,
        TrustRank::KnownUser => HighRiskRank::KnownUser,
        TrustRank::TrustedUser => HighRiskRank::TrustedUser,
    };
    nickname_matches(settings.nickname, display_name)
        && (settings.ranks.contains(&rank)
            || (trust.is_probable_troll && settings.marks.contains(&HighRiskMark::ProbableTroll))
            || (trust.is_troll && settings.marks.contains(&HighRiskMark::Troll)))
}

async fn lookup(app: &AppHandle, user_id: &str) -> Option<Profile> {
    let state = app.state::<AppState>();
    let response = state
        .runtime_host()
        .get_user_via_cache(user_id.to_string(), false, false, None)
        .await
        .ok()?;
    if !(200..300).contains(&response.status) {
        return None;
    }
    let profile: Profile = serde_json::from_str(&response.data).ok()?;
    (profile.id == user_id).then_some(profile)
}

enum ScanStatus {
    Running,
    Checked,
    RetryAt(Instant),
}

fn same_rules(a: &HighRiskDetectionSettings, b: &HighRiskDetectionSettings) -> bool {
    a.rooms == b.rooms && a.nickname == b.nickname && a.ranks == b.ranks && a.marks == b.marks
}

async fn run(app: AppHandle) {
    let state = app.state::<AppState>();
    let shared = app.state::<Shared>();
    let mut ticker = tokio::time::interval(SCAN_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut lookups = tokio::task::JoinSet::new();
    let mut scanned = HashMap::<Encounter, ScanStatus>::new();
    let mut previous_room = None;
    let mut previous_scope = None;
    let mut previous_settings = HighRiskDetectionSettings::default();
    let mut scan_id = 0_u64;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let scope = state.runtime_host().auth_scope_snapshot();
                let settings = {
                    let mut detection = shared.lock();
                    detection.refresh_scope(&scope);
                    detection.settings.clone()
                };
                let (room, players) = if scope.active {
                    current_room(&state, &settings)
                } else {
                    (None, Vec::new())
                };
                if previous_room != room || !same_scope(previous_scope.as_ref(), &scope)
                    || !same_rules(&previous_settings, &settings)
                {
                    lookups.abort_all();
                    scanned.clear();
                    scan_id += 1;
                }
                previous_room = room;
                previous_scope = Some(scope.clone());
                previous_settings = settings.clone();
                let present: HashSet<_> = players.iter().map(Encounter::from).collect();
                scanned.retain(|player, _| present.contains(player));
                for player in &players {
                    if lookups.len() >= MAX_LOOKUPS { break; }
                    if player.user_id.is_empty() || player.user_id == scope.current_user_id
                        || !(player.display_name.is_empty()
                            || nickname_matches(settings.nickname, &player.display_name))
                    {
                        continue;
                    }
                    let encounter = Encounter::from(player);
                    match scanned.get(&encounter) {
                        None => {}
                        Some(ScanStatus::RetryAt(time)) if *time <= Instant::now() => {}
                        _ => continue,
                    }
                    scanned.insert(encounter.clone(), ScanStatus::Running);
                    let app = app.clone();
                    lookups.spawn(async move {
                        let profile = lookup(&app, &encounter.user_id).await;
                        (scan_id, encounter, profile)
                    });
                }
            }
            Some(Ok((id, encounter, profile))) = lookups.join_next(), if !lookups.is_empty() => {
                if id != scan_id || !matches!(scanned.get(&encounter), Some(ScanStatus::Running)) {
                    continue;
                }
                let Some(profile) = profile else {
                    scanned.insert(encounter, ScanStatus::RetryAt(Instant::now() + LOOKUP_RETRY));
                    continue;
                };
                let scope = state.runtime_host().auth_scope_snapshot();
                if !scope.active || !same_scope(previous_scope.as_ref(), &scope) { continue; }
                let settings = shared.lock().settings.clone();
                if !same_rules(&previous_settings, &settings) { continue; }
                let (room, players) = current_room(&state, &settings);
                if room != previous_room { continue; }
                let Some(player) = players.iter().find(|player| Encounter::from(*player) == encounter) else {
                    scanned.remove(&encounter);
                    continue;
                };
                let display_name = if player.display_name.is_empty() {
                    profile.display_name
                } else {
                    player.display_name.clone()
                };
                if profile_matches(&settings, &display_name, &profile.tags) {
                    let mut detection = shared.lock();
                    if same_rules(&detection.settings, &settings)
                        && same_scope(detection.scope.as_ref(), &scope)
                        && !detection.pending.contains_key(&encounter.user_id)
                    {
                        if detection.pending.is_empty() {
                            detection.next_alert = Some(Instant::now());
                        }
                        detection.pending.insert(encounter.user_id.clone(), PendingPlayer {
                            display_name,
                            blocked: false,
                        });
                    }
                }
                scanned.insert(encounter, ScanStatus::Checked);
            }
        }
        alert_if_due(&app, &shared, &state).await;
    }
}

async fn alert_if_due(app: &AppHandle, shared: &Shared, state: &AppState) {
    let (clear_shortcut, alert) = {
        let mut detection = shared.lock();
        let clear_shortcut = detection.pending.is_empty() && detection.active_shortcut.is_some();
        let alert = if !detection.processing
            && !detection.pending.is_empty()
            && detection
                .next_alert
                .is_some_and(|time| time <= Instant::now())
        {
            detection.toast_id += 1;
            detection.next_alert = Some(Instant::now() + TOAST_FALLBACK_INTERVAL);
            Some(detection.toast_id)
        } else {
            None
        };
        (clear_shortcut, alert)
    };
    if clear_shortcut || alert.is_some() {
        let available =
            tray_shortcut::on_main_thread(app, move |app| sync_shortcut(app, alert.is_some()))
                .await
                .unwrap_or(false);
        if let Some(toast_id) = alert {
            let (names, shortcut) = {
                let detection = shared.lock();
                if detection.processing
                    || detection.pending.is_empty()
                    || detection.toast_id != toast_id
                {
                    return;
                }
                let names = detection
                    .pending
                    .values()
                    .take(3)
                    .map(|player| player.display_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let extra = detection.pending.len().saturating_sub(3);
                let names = if extra == 0 {
                    names
                } else {
                    format!("{names} (+{extra})")
                };
                (names, shortcut_label(&detection.settings.shortcut))
            };
            let chinese = app_language(state).starts_with("zh");
            let (title, message) = match (chinese, available) {
                (true, true) => ("高危人员检测", format!("{names}。按 {shortcut} 处理。")),
                (true, false) => (
                    "高危人员检测",
                    format!("{names}。{shortcut} 不可用，请在通知设置中更改处理快捷键。"),
                ),
                (false, true) => (
                    "High-risk player detected",
                    format!("{names}. Press {shortcut} to respond."),
                ),
                (false, false) => (
                    "High-risk player detected",
                    format!(
                        "{names}. {shortcut} is unavailable. Change it in notification settings."
                    ),
                ),
            };
            show_alert(app, title, &message, toast_id);
        }
    }
}

fn shortcut_label(binding: &TrayShortcutBinding) -> String {
    binding
        .accelerator()
        .replace("Control+", "Ctrl+")
        .replace("Key", "")
        .replace("Digit", "")
}

fn toast_finished(app: &AppHandle, toast_id: u64) {
    if let Some(shared) = app.try_state::<Shared>() {
        let mut detection = shared.lock();
        if detection.toast_id == toast_id && !detection.pending.is_empty() {
            detection.next_alert = Some(Instant::now() + SCAN_INTERVAL);
        }
    }
}

fn show_alert(app: &AppHandle, title: &str, message: &str, toast_id: u64) {
    let dismissed = app.clone();
    let activated = app.clone();
    let result = windows_notification_app_id(app).and_then(|app_id| {
        Toast::new(&app_id)
            .title(title)
            .text2(message)
            .duration(tauri_winrt_notification::Duration::Short)
            .sound(Some(Sound::Default))
            .on_dismissed(move |_| {
                toast_finished(&dismissed, toast_id);
                Ok(())
            })
            .on_activated(move |_| {
                toast_finished(&activated, toast_id);
                Ok(())
            })
            .show()
            .map_err(|error| error.to_string())
    });
    if let Err(error) = result {
        tracing::warn!(%error, "Failed to show the high-risk player alert");
    }
}

async fn process_pending(app: &AppHandle, shortcut: Shortcut) {
    let state = app.state::<AppState>();
    let host = state.runtime_host();
    let shared = app.state::<Shared>();
    let scope = host.auth_scope_snapshot();
    let (players, auto_block) = {
        let mut detection = shared.lock();
        detection.refresh_scope(&scope);
        if !scope.active
            || detection.processing
            || detection.pending.is_empty()
            || detection.active_shortcut != Some(shortcut)
        {
            return;
        }
        detection.processing = true;
        (detection.pending.clone(), detection.settings.auto_block)
    };

    let mut blocked = HashSet::new();
    if auto_block {
        for (id, player) in &players {
            if !same_scope(Some(&scope), &host.auth_scope_snapshot()) {
                break;
            }
            let already_blocked = player.blocked
                || host
                    .local_data()
                    .local_moderation_get(OwnerId::new(scope.current_user_id.clone()), id.clone())
                    .ok()
                    .flatten()
                    .is_some_and(|row| row.block);
            if already_blocked {
                blocked.insert(id.clone());
                continue;
            }
            let input = ModerationSyncMutationInput::block(id.clone(), player.display_name.clone());
            match host.social().moderation_update(input).await {
                Ok(_) => {
                    blocked.insert(id.clone());
                }
                Err(error) => tracing::warn!(%error, "Failed to block a high-risk player"),
            }
        }
    }

    let avatar_selected = if same_scope(Some(&scope), &host.auth_scope_snapshot()) {
        match host.avatars().select(ROBOT_AVATAR_ID.into()).await {
            Ok(result) if (200..300).contains(&result.response.status) => true,
            Ok(result) => {
                tracing::warn!(
                    status = result.response.status,
                    "Failed to switch to the robot avatar"
                );
                false
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to switch to the robot avatar");
                false
            }
        }
    } else {
        false
    };

    let complete = {
        let mut detection = shared.lock();
        if !same_scope(Some(&scope), &host.auth_scope_snapshot())
            || !same_scope(detection.scope.as_ref(), &scope)
        {
            return;
        }
        let mut complete = avatar_selected;
        for id in players.keys() {
            if avatar_selected && (!auto_block || blocked.contains(id)) {
                detection.pending.remove(id);
            } else if let Some(player) = detection.pending.get_mut(id) {
                player.blocked |= blocked.contains(id);
                complete = false;
            }
        }
        detection.processing = false;
        if detection.pending.is_empty() {
            detection.next_alert = None;
            detection.toast_id += 1;
        }
        complete
    };
    if let Err(error) = tray_shortcut::on_main_thread(app, |app| sync_shortcut(app, false)).await {
        tracing::warn!(%error, "Failed to update the high-risk handling shortcut");
    }
    if !same_scope(Some(&scope), &host.auth_scope_snapshot()) {
        return;
    }
    let chinese = app_language(&state).starts_with("zh");
    let (title, message) = match (chinese, complete) {
        (true, true) => ("高危人员处理完毕", "已切换为官方机器人 Avatar。"),
        (true, false) => ("高危人员处理未完成", "部分操作失败，请按处理快捷键重试。"),
        (false, true) => (
            "High-risk players handled",
            "Switched to the official robot avatar.",
        ),
        (false, false) => (
            "High-risk response incomplete",
            "Some actions failed. Press the handling shortcut to retry.",
        ),
    };
    if let Err(error) = windows_notification_app_id(app).and_then(|app_id| {
        Toast::new(&app_id)
            .title(title)
            .text2(message)
            .sound(None)
            .show()
            .map_err(|e| e.to_string())
    }) {
        tracing::warn!(%error, "Failed to show the high-risk response result");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nickname_is_required_for_both_rank_and_mark() {
        let mut settings = HighRiskDetectionSettings::default();
        settings.marks.push(HighRiskMark::Troll);
        let troll = ["system_trust_veteran".into(), "system_troll".into()];
        assert!(profile_matches(&settings, "Player a1b2", &[]));
        assert!(profile_matches(&settings, "Player", &troll));
        assert!(!profile_matches(&settings, "玩家 a1b2", &troll));
        assert!(!profile_matches(
            &settings,
            "Player",
            &["system_trust_veteran".into()]
        ));
        settings.nickname = HighRiskNickname::Suffix;
        assert!(profile_matches(&settings, "玩家 a1b2", &troll));
        assert!(!profile_matches(&settings, "Player", &troll));
        settings.nickname = HighRiskNickname::Ignore;
        assert!(profile_matches(&settings, "玩家", &troll));
    }

    #[test]
    fn room_categories_distinguish_access_modifiers() {
        for (suffix, kind) in [
            ("", Some(HighRiskRoom::Public)),
            (
                "~group(grp_x)~groupAccessType(public)",
                Some(HighRiskRoom::GroupPublic),
            ),
            (
                "~group(grp_x)~groupAccessType(plus)",
                Some(HighRiskRoom::GroupPlus),
            ),
            ("~group(grp_x)~groupAccessType(members)", None),
            ("~hidden(usr_x)", Some(HighRiskRoom::FriendsPlus)),
            ("~friends(usr_x)", Some(HighRiskRoom::Friends)),
            (
                "~private(usr_x)~canRequestInvite",
                Some(HighRiskRoom::InvitePlus),
            ),
            ("~private(usr_x)", Some(HighRiskRoom::Invite)),
        ] {
            assert_eq!(room_kind(&format!("wrld_sample:123{suffix}")), kind);
        }
        assert_eq!(room_kind("traveling"), None);
    }
}
