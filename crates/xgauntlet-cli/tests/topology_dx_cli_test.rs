use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn make_temp_workspace(prefix: &str) -> PathBuf {
    let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "xgauntlet_topology_dx_cli_{prefix}_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src/features/billing")).unwrap();
    fs::create_dir_all(dir.join("src/features/shipping")).unwrap();
    fs::create_dir_all(dir.join("tasks")).unwrap();

    fs::write(
        dir.join("gauntlet.toml"),
        "stack = \"rust\"\n[paths]\ntasks_dir = \"tasks\"\n",
    )
    .unwrap();
    fs::write(
        dir.join("src/features/billing/models.rs"),
        "/// @domain\npub struct Invoice { pub id: String }\n",
    )
    .unwrap();
    fs::write(
        dir.join("src/features/shipping/models.rs"),
        "/// @domain\npub struct Shipment { pub code: String }\n",
    )
    .unwrap();
    dir
}

#[test]
fn test_cli_topology_export_with_feature_filter() {
    let ws = make_temp_workspace("export_feature");
    let bin = env!("CARGO_BIN_EXE_xgauntlet");

    let out = Command::new(bin)
        .args([
            "topology",
            "export",
            "--format",
            "mermaid",
            "--feature",
            "billing",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("execute topology export");

    assert!(
        out.status.success(),
        "topology export --feature billing failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Invoice"));
    assert!(!stdout.contains("Shipment"));

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn test_cli_topology_open_no_browser_writes_pele_viewer_html() {
    let ws = make_temp_workspace("open_viewer");
    let out_html = ws.join(".xgauntlet/custom-viewer.html");
    let bin = env!("CARGO_BIN_EXE_xgauntlet");

    let out = Command::new(bin)
        .args([
            "topology",
            "open",
            "--feature",
            "billing",
            "--no-browser",
            "--out",
            out_html.to_str().unwrap(),
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("execute topology open");

    assert!(
        out.status.success(),
        "topology open --no-browser failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out_html.is_file(), "viewer HTML must be written");
    let html = fs::read_to_string(&out_html).unwrap();
    assert!(html.contains("Invoice"));
    assert!(!html.contains("Shipment"));
    assert!(html.contains("pele"));

    let _ = fs::remove_dir_all(&ws);
}

fn _keep_path_import_clean(_p: &Path) {}
