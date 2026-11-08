//! Acceptance and invariant tests for Task 071:
//! Domain AST Field & Relationship Extraction for Mermaid.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::topology::{
    export_mermaid_class_diagram, scan_workspace_topology, MermaidExportOptions, MermaidLevel,
    TopologyOptions,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(format!("xgauntlet_field_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_ast_scans_struct_fields_enum_variants_and_relationships() {
    let ws = TempWorkspace::new("domain_ast");
    let ws_path = ws.path();

    // Setup minimal Cargo workspace
    fs::write(
        ws_path.join("Cargo.toml"),
        r#"[package]
name = "shop-core"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let models_dir = ws_path.join("src").join("features").join("order");
    fs::create_dir_all(&models_dir).unwrap();

    let models_rs = models_dir.join("models.rs");
    fs::write(
        &models_rs,
        r#"//! Domain models for order slice.

pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

pub struct Customer {
    pub id: String,
    pub name: String,
}

pub struct Order {
    pub order_id: String,
    pub customer: Customer,
    pub status: PaymentStatus,
    pub items: Vec<String>,
}
"#,
    )
    .unwrap();

    let topology_opts = TopologyOptions {
        workspace_root: ws_path.to_path_buf(),
        detect_symbols: true,
        ..Default::default()
    };

    let graph = scan_workspace_topology(&topology_opts)
        .expect("Topology scan should succeed on valid workspace");

    let export_opts = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };

    let mermaid = export_mermaid_class_diagram(&graph, &export_opts);

    // 1. Verify Enum variants extraction
    assert!(
        mermaid.contains("class PaymentStatus {"),
        "Diagram must contain class PaymentStatus:\n{mermaid}"
    );
    assert!(
        mermaid.contains("<<enumeration>>"),
        "PaymentStatus must be an enumeration:\n{mermaid}"
    );
    assert!(
        mermaid.contains("Pending") && mermaid.contains("Completed") && mermaid.contains("Failed"),
        "PaymentStatus must declare variants Pending, Completed, Failed:\n{mermaid}"
    );

    // 2. Verify Struct fields extraction
    assert!(
        mermaid.contains("class Customer {"),
        "Diagram must contain class Customer:\n{mermaid}"
    );
    assert!(
        mermaid.contains("+id: String") && mermaid.contains("+name: String"),
        "Customer must list fields id and name:\n{mermaid}"
    );

    assert!(
        mermaid.contains("class Order {"),
        "Diagram must contain class Order:\n{mermaid}"
    );
    assert!(
        mermaid.contains("+order_id: String")
            && mermaid.contains("+customer: Customer")
            && mermaid.contains("+status: PaymentStatus"),
        "Order must list fields order_id, customer, and status:\n{mermaid}"
    );

    // 3. Verify Inter-Model Relationship extraction
    assert!(
        mermaid.contains("Order *-- Customer : customer")
            || mermaid.contains("Order --> Customer : customer"),
        "Diagram must show relation between Order and Customer with field label 'customer':\n{mermaid}"
    );
    assert!(
        mermaid.contains("Order *-- PaymentStatus : status")
            || mermaid.contains("Order --> PaymentStatus : status"),
        "Diagram must show relation between Order and PaymentStatus with field label 'status':\n{mermaid}"
    );
}
