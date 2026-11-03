//! The manifest, for a tab that is native: schema 2, the `entry` is a `.wasm`
//! file that exists and lies inside the kit, and is a module Hangar loads
//! (WebAssembly that imports nothing but `hangar.log` and `hangar.now`).

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

fn kit() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn manifest() -> Value {
    let text = std::fs::read_to_string(kit().join("hangar-kit.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn tab() -> Value {
    manifest()["tabs"][0].clone()
}

#[test]
fn the_manifest_is_schema_2_and_the_entry_is_the_module() {
    let manifest = manifest();
    assert_eq!(manifest["schema"], 2);
    assert_eq!(manifest["hangar"], ">=2.0.0");
    let tab = tab();
    assert_eq!(tab["entry"], "tabs/todo.wasm");
    assert!(tab.get("native").is_none(), "schema 2 has no `native` field");
    assert!(!kit().join("tabs/todo").exists(), "no HTML page is left");
}

#[test]
fn the_permissions_are_those_of_the_page() {
    let mut permissions: Vec<String> = tab()["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_owned())
        .collect();
    permissions.sort();
    assert_eq!(
        permissions,
        [
            "command:scan",
            "data.read",
            "data.write",
            "panes.read",
            "panes.send",
            "preview.show"
        ]
    );
}

#[test]
fn the_entry_names_a_wasm_file_inside_the_kit() {
    let tab = tab();
    let native = Path::new(tab["entry"].as_str().unwrap());
    assert_eq!(native.extension().and_then(|e| e.to_str()), Some("wasm"));
    // Relative, inside the kit, nothing hidden.
    assert!(native.is_relative());
    for part in native.components() {
        match part {
            Component::Normal(name) => assert!(!name.to_str().unwrap().starts_with('.')),
            other => panic!("{other:?} leaves the kit"),
        }
    }
    let file = kit().join(native);
    assert!(
        file.is_file(),
        "{} is not built: scripts/build-native.sh",
        file.display()
    );
    let size = std::fs::metadata(&file).unwrap().len();
    assert!(size > 0 && size <= 64 * 1024 * 1024, "{size} bytes");
}

#[test]
fn the_module_is_webassembly_for_hangar_and_not_for_wasi_or_wasm_bindgen() {
    let tab = tab();
    let bytes = std::fs::read(kit().join(tab["entry"].as_str().unwrap())).unwrap();
    assert_eq!(&bytes[..4], b"\0asm");
    let has = |needle: &str| bytes.windows(needle.len()).any(|w| w == needle.as_bytes());
    for export in [
        "memory",
        "hangar_abi_version",
        "hangar_alloc",
        "hangar_init",
        "hangar_update",
        "hangar_view",
    ] {
        assert!(has(export), "no export {export}");
    }
    assert!(
        !has("wasi_snapshot_preview1"),
        "built for WASI: use wasm32-unknown-unknown"
    );
    assert!(!has("__wbindgen"), "built with wasm-bindgen");
}

#[test]
fn nothing_personal_is_left_in_the_module() {
    // The build remaps the paths of the machine it ran on.
    let tab = tab();
    let bytes = std::fs::read(kit().join(tab["entry"].as_str().unwrap())).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    // Written in pieces, so that this file does not hold them itself (a test
    // looks for them in every file of the kit).
    let forbidden = [
        format!("/{}/", "home"),
        format!("/{}/", "Users"),
        format!("C:{}{}", '\\', "Users"),
        format!(".cargo/{}", "registry"),
    ];
    for forbidden in forbidden {
        assert!(!text.contains(&forbidden), "the module holds {forbidden}");
    }
}
