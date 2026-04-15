use logtok::store::Store;
use logtok::tokenizer::{TokenEntry, TokenMapData};
use std::sync::Mutex;
use tempfile::TempDir;

// Serialize all store tests since they share the LOGTOK_KEY env var
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn sample_token_map_data() -> TokenMapData {
    let mut data = TokenMapData::default();
    data.value_to_token
        .insert("192.168.1.1".to_string(), "[IP_001]".to_string());
    data.token_to_value
        .insert("[IP_001]".to_string(), "192.168.1.1".to_string());
    data.category_counters.insert("IP".to_string(), 1);
    data.entries.insert(
        "192.168.1.1".to_string(),
        TokenEntry {
            token: "[IP_001]".to_string(),
            category: "IP".to_string(),
            created_at: 1700000000,
        },
    );
    data
}

#[test]
fn save_then_load_round_trips_data() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "test-passphrase-123") };
    let store = Store::new(&store_dir).unwrap();

    let original = sample_token_map_data();
    store.save(&original).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded.value_to_token, original.value_to_token);
    assert_eq!(loaded.token_to_value, original.token_to_value);
    assert_eq!(loaded.category_counters, original.category_counters);
    assert_eq!(loaded.entries.len(), original.entries.len());
}

#[test]
fn store_file_has_magic_bytes_and_version() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "test-passphrase-magic") };
    let store = Store::new(&store_dir).unwrap();

    store.save(&sample_token_map_data()).unwrap();

    let raw = std::fs::read(store_dir.join("store.enc")).unwrap();
    assert_eq!(&raw[0..4], b"LTOK", "Magic bytes should be LTOK");
    assert_eq!(raw[4], 0x01, "Version should be 0x01");
}

#[test]
fn same_passphrase_same_salt_derives_same_key() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "consistent-key-test") };
    let store = Store::new(&store_dir).unwrap();

    let data = sample_token_map_data();
    store.save(&data).unwrap();

    // Load with same passphrase should work
    let store2 = Store::new(&store_dir).unwrap();
    let loaded = store2.load().unwrap();
    assert_eq!(loaded.value_to_token, data.value_to_token);
}

#[test]
fn wrong_passphrase_fails_to_decrypt() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "correct-passphrase") };
    let store = Store::new(&store_dir).unwrap();
    store.save(&sample_token_map_data()).unwrap();

    // Try loading with different passphrase
    unsafe { std::env::set_var("LOGTOK_KEY", "wrong-passphrase") };
    let store2 = Store::new(&store_dir).unwrap();
    let result = store2.load();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Decryption failed") || err_msg.contains("wrong LOGTOK_KEY"),
        "Error should mention decryption failure, got: {}",
        err_msg
    );
}

#[test]
fn each_save_generates_different_nonce() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "nonce-test-key") };
    let store = Store::new(&store_dir).unwrap();

    let data = sample_token_map_data();
    store.save(&data).unwrap();
    let raw1 = std::fs::read(store_dir.join("store.enc")).unwrap();

    store.save(&data).unwrap();
    let raw2 = std::fs::read(store_dir.join("store.enc")).unwrap();

    // Nonce is at bytes 21..33 (after 4 magic + 1 version + 16 salt)
    let nonce1 = &raw1[21..33];
    let nonce2 = &raw2[21..33];
    assert_ne!(nonce1, nonce2, "Each save must generate a fresh nonce");
}

#[test]
fn reset_deletes_store_file() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "reset-test-key") };
    let store = Store::new(&store_dir).unwrap();

    store.save(&sample_token_map_data()).unwrap();
    assert!(store_dir.join("store.enc").exists());

    store.reset().unwrap();
    assert!(!store_dir.join("store.enc").exists());
}

#[test]
fn load_nonexistent_file_returns_empty_data() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "empty-load-key") };
    let store = Store::new(&store_dir).unwrap();

    let data = store.load().unwrap();
    assert!(data.value_to_token.is_empty());
    assert!(data.token_to_value.is_empty());
    assert!(data.category_counters.is_empty());
    assert!(data.entries.is_empty());
}

#[test]
fn missing_logtok_key_produces_error() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::remove_var("LOGTOK_KEY") };
    let result = Store::new(&store_dir);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("LOGTOK_KEY"),
        "Error should mention LOGTOK_KEY, got: {}",
        err_msg
    );
    // Restore env var so other tests aren't affected
    unsafe { std::env::set_var("LOGTOK_KEY", "restored-after-test") };
}

#[test]
fn store_creates_directory_if_missing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join("deeply").join("nested").join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "dir-create-test") };
    let store = Store::new(&store_dir).unwrap();

    store.save(&sample_token_map_data()).unwrap();
    assert!(store_dir.join("store.enc").exists());
}

#[test]
fn atomic_write_uses_temp_file() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Verify that the final store.enc exists but no .enc.tmp remains after save
    let dir = TempDir::new().unwrap();
    let store_dir = dir.path().join(".logtok");

    unsafe { std::env::set_var("LOGTOK_KEY", "atomic-write-test") };
    let store = Store::new(&store_dir).unwrap();

    store.save(&sample_token_map_data()).unwrap();

    assert!(store_dir.join("store.enc").exists());
    assert!(
        !store_dir.join("store.enc.tmp").exists(),
        "Temp file should be renamed away after save"
    );
}
