use std::collections::HashMap;
use std::path::PathBuf;

use matrix_mcp::config::{Config, ConfigError};

fn load(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
    let env: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Config::from_lookup(|key| env.get(key).cloned(), PathBuf::from("/home/u"))
}

#[test]
fn resolves_labels_and_bare_ids() {
    let config = load(&[("MATRIX_ROOM_WHITELIST", "self=!a:x.org, !b:y.org")]).unwrap();
    let whitelist = &config.room_whitelist;

    assert_eq!(whitelist.resolve("self").unwrap(), "!a:x.org");
    assert_eq!(whitelist.resolve("!a:x.org").unwrap(), "!a:x.org");
    assert_eq!(whitelist.resolve("!b:y.org").unwrap(), "!b:y.org");
    assert!(whitelist.resolve("other").is_none());
}

#[test]
fn matches_labels_ignoring_case() {
    let config = load(&[("MATRIX_ROOM_WHITELIST", "Self=!a:x.org")]).unwrap();

    assert_eq!(config.room_whitelist.resolve("self").unwrap(), "!a:x.org");
}

#[test]
fn refuses_duplicate_labels_ignoring_case() {
    let err = load(&[("MATRIX_ROOM_WHITELIST", "self=!a:x.org,SELF=!b:x.org")]).unwrap_err();

    assert!(err.to_string().contains("SELF"), "{err}");
}

#[test]
fn refuses_empty_label() {
    let err = load(&[("MATRIX_ROOM_WHITELIST", " =!a:x.org")]).unwrap_err();

    assert!(err.to_string().contains("empty label"), "{err}");
}

#[test]
fn refuses_missing_or_empty_whitelist() {
    for vars in [&[][..], &[("MATRIX_ROOM_WHITELIST", " , ")][..]] {
        let err = load(vars).unwrap_err();
        assert!(err.to_string().contains("MATRIX_ROOM_WHITELIST"), "{err}");
    }
}

#[test]
fn refuses_aliases_in_whitelist() {
    let err = load(&[("MATRIX_ROOM_WHITELIST", "!a:x.org,#alias:x.org")]).unwrap_err();

    assert!(err.to_string().contains("#alias:x.org"), "{err}");
}

#[test]
fn defaults_dir_to_config_matrix_mcp() {
    let config = load(&[("MATRIX_ROOM_WHITELIST", "!a:x.org")]).unwrap();

    assert_eq!(config.dir, PathBuf::from("/home/u/.config/matrix-mcp"));
}
