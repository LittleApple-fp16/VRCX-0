use super::storage::AuthCredentialStore;

use super::storage::{
    get_config_bool, get_config_string, read_saved_credentials, remove_config_value,
    write_saved_credentials, LAST_USER_LOGGED_IN_KEY,
};
use super::types::{
    SavedAuthSnapshot, SavedCredential, SavedCredentialSnapshot, SavedCredentials,
    SavedLoginParamsSnapshot,
};
use vrcx_0_application_core::Result;

const LEGACY_PRIMARY_PASSWORD_KEY: &str = "enablePrimaryPassword";

pub fn saved_snapshot(config: &dyn AuthCredentialStore) -> Result<SavedAuthSnapshot> {
    build_saved_auth_snapshot(config)
}

impl SavedCredential {
    fn has_login_credentials(&self) -> bool {
        !self.login_params.username.trim().is_empty()
            && self
                .login_params
                .password
                .as_deref()
                .is_some_and(|password| !password.trim().is_empty())
    }

    fn has_cookies(&self) -> bool {
        self.cookies
            .as_deref()
            .is_some_and(|cookies| !cookies.is_empty())
    }

    fn redacted(&self) -> SavedCredentialSnapshot {
        SavedCredentialSnapshot {
            user: self.user.clone(),
            login_params: SavedLoginParamsSnapshot {
                username: self.login_params.username.clone(),
            },
            has_login_credentials: self.has_login_credentials(),
            has_cookies: self.has_cookies(),
        }
    }
}

fn saved_credential_sort_name(value: &SavedCredential) -> String {
    value
        .user
        .display_name
        .as_deref()
        .filter(|display_name| !display_name.trim().is_empty())
        .or(value.user.username.as_deref())
        .unwrap_or_default()
        .trim()
        .to_lowercase()
}

fn sorted_redacted_saved_credentials_list(
    saved_credentials: &SavedCredentials,
    last_user_logged_in: &str,
) -> Vec<SavedCredentialSnapshot> {
    let mut values = saved_credentials.values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        let left_is_last = !last_user_logged_in.is_empty() && left.user.id == last_user_logged_in;
        let right_is_last = !last_user_logged_in.is_empty() && right.user.id == last_user_logged_in;
        if left_is_last != right_is_last {
            return right_is_last.cmp(&left_is_last);
        }
        saved_credential_sort_name(left).cmp(&saved_credential_sort_name(right))
    });
    values.into_iter().map(SavedCredential::redacted).collect()
}

pub(super) fn build_saved_auth_snapshot(
    config: &dyn AuthCredentialStore,
) -> Result<SavedAuthSnapshot> {
    let mut saved_credentials = read_saved_credentials(config)?;
    let mut last_user_logged_in = get_config_string(config, LAST_USER_LOGGED_IN_KEY, "")?;
    let legacy_primary_password_enabled =
        get_config_bool(config, LEGACY_PRIMARY_PASSWORD_KEY, false)?;
    if legacy_primary_password_enabled {
        saved_credentials.clear();
        last_user_logged_in.clear();
        write_saved_credentials(config, &saved_credentials)?;
        remove_config_value(config, LEGACY_PRIMARY_PASSWORD_KEY)?;
        remove_config_value(config, LAST_USER_LOGGED_IN_KEY)?;
    }

    let saved_credentials_list =
        sorted_redacted_saved_credentials_list(&saved_credentials, &last_user_logged_in);

    Ok(SavedAuthSnapshot {
        last_user_logged_in: (!last_user_logged_in.is_empty()).then_some(last_user_logged_in),
        saved_credentials_list,
    })
}
