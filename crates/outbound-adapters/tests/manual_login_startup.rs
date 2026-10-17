use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use vrcx_0_outbound_adapters::LocalWebClientAdapter as WebClient;
use vrcx_0_persistence::{cookies, storage::StorageService, DatabaseService};

#[test]
fn startup_does_not_restore_global_cookies_but_manual_selection_can_restore_them() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("manual-login-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = Arc::new(DatabaseService::new(&dir.join("test.sqlite3")).unwrap());
    let storage = StorageService::new(&dir.join("storage.json")).unwrap();
    let saved = B64.encode(
        serde_json::json!([{
            "Name": "auth",
            "Value": "previous-session",
            "Domain": ".vrchat.cloud",
            "Path": "/"
        }])
        .to_string(),
    );
    cookies::save_default_cookies(&db, &saved).unwrap();

    let web = WebClient::new(&storage, Arc::clone(&db), "http://localhost".into(), "test").unwrap();
    assert!(web.auth_cookie_value().is_none());
    assert_eq!(web.cookie_diagnostics()["cookieCount"], 0);

    // The saved account path remains available once the user explicitly selects it.
    web.set_cookies(&saved).unwrap();
    assert_eq!(web.auth_cookie_value().as_deref(), Some("previous-session"));
    drop(web);
    let restarted =
        WebClient::new(&storage, Arc::clone(&db), "http://localhost".into(), "test").unwrap();
    assert!(restarted.auth_cookie_value().is_none());
    drop(restarted);
    drop(storage);
    drop(db);
    std::fs::remove_dir_all(dir).unwrap();
}
