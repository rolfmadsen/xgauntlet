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

#[test]
fn test_ast_scans_doc_commented_struct_fields_enum_variants_and_relationships() {
    let ws = TempWorkspace::new("doc_comments");
    let ws_path = ws.path();

    fs::write(
        ws_path.join("Cargo.toml"),
        r#"[package]
name = "doctor-core"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let models_dir = ws_path.join("src").join("features").join("doctor");
    fs::create_dir_all(&models_dir).unwrap();

    let models_rs = models_dir.join("models.rs");
    fs::write(
        &models_rs,
        r#"//! Domain models for doctor diagnostics.

/// Categories for diagnostic checks.
pub enum DoctorCategory {
    /// Host platform metadata.
    Host,
    /// Git repository checks.
    Git,
}

/// Execution status of an individual diagnostic check.
pub enum DoctorCheckStatus {
    /// Check passed.
    Pass,
    /// Check failed.
    Fail,
}

/// Result of an individual diagnostic check.
pub struct DoctorCheckItem {
    /// Identifier name of the check.
    pub name: String,
    /// Category grouping for the check.
    pub category: DoctorCategory,
    /// Final status of this check.
    pub status: DoctorCheckStatus,
    /// Execution duration.
    pub duration_ms: u64,
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

    // 1. Verify Enum variants with doc comments
    assert!(
        mermaid.contains("Host") && mermaid.contains("Git"),
        "DoctorCategory must declare variants Host and Git:\n{mermaid}"
    );
    assert!(
        mermaid.contains("Pass") && mermaid.contains("Fail"),
        "DoctorCheckStatus must declare variants Pass and Fail:\n{mermaid}"
    );

    // 2. Verify Struct fields with doc comments
    assert!(
        mermaid.contains("+name: String")
            && mermaid.contains("+category: DoctorCategory")
            && mermaid.contains("+status: DoctorCheckStatus")
            && mermaid.contains("+duration_ms: u64"),
        "DoctorCheckItem must extract all fields even when preceded by doc comments:\n{mermaid}"
    );

    // 3. Verify Relationships derived from doc-commented fields
    assert!(
        mermaid.contains("DoctorCheckItem *-- DoctorCategory : category"),
        "Diagram must show relation DoctorCheckItem *-- DoctorCategory : category:\n{mermaid}"
    );
    assert!(
        mermaid.contains("DoctorCheckItem *-- DoctorCheckStatus : status"),
        "Diagram must show relation DoctorCheckItem *-- DoctorCheckStatus : status:\n{mermaid}"
    );
}

#[test]
fn test_conceptual_mermaid_filters_errors_and_options_while_detailed_keeps_them() {
    let ws = TempWorkspace::new("filter_test");
    let ws_path = ws.path();

    fs::write(
        ws_path.join("Cargo.toml"),
        r#"[package]
name = "filter-core"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let models_dir = ws_path.join("src").join("features").join("billing");
    fs::create_dir_all(&models_dir).unwrap();

    let models_rs = models_dir.join("models.rs");
    fs::write(
        &models_rs,
        r#"//! Billing domain models and scaffolding.

pub enum InvoiceStatus {
    Draft,
    Paid,
}

pub struct Invoice {
    pub id: String,
    pub status: InvoiceStatus,
}

pub enum InvoiceError {
    NotFound,
    PaymentFailed,
}

pub struct InvoiceOptions {
    pub dry_run: bool,
}
"#,
    )
    .unwrap();

    let topology_opts = TopologyOptions {
        workspace_root: ws_path.to_path_buf(),
        detect_symbols: true,
        ..Default::default()
    };

    let graph = scan_workspace_topology(&topology_opts).unwrap();

    // 1. Conceptual export: MUST filter out InvoiceError and InvoiceOptions and have header
    let export_opts_conceptual = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };
    let mermaid_conceptual = export_mermaid_class_diagram(&graph, &export_opts_conceptual);

    assert!(
        mermaid_conceptual.contains("class Invoice {"),
        "Conceptual diagram must contain business entity Invoice:\n{mermaid_conceptual}"
    );
    assert!(
        mermaid_conceptual.contains("class InvoiceStatus {"),
        "Conceptual diagram must contain business enum InvoiceStatus:\n{mermaid_conceptual}"
    );
    assert!(
        mermaid_conceptual.contains("Invoice *-- InvoiceStatus : status"),
        "Conceptual diagram must contain composition relationship:\n{mermaid_conceptual}"
    );
    assert!(
        mermaid_conceptual.contains("%% 🏛️ xGauntlet Core Domain Model"),
        "Conceptual diagram must declare conceptual domain header comment:\n{mermaid_conceptual}"
    );
    assert!(
        !mermaid_conceptual.contains("class InvoiceError"),
        "Conceptual diagram must NOT include technical error enum InvoiceError:\n{mermaid_conceptual}"
    );
    assert!(
        !mermaid_conceptual.contains("class InvoiceOptions"),
        "Conceptual diagram must NOT include technical CLI options InvoiceOptions:\n{mermaid_conceptual}"
    );

    // 2. Detailed export: MUST retain all types
    let export_opts_detailed = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Detailed,
        domain_only: false,
    };
    let mermaid_detailed = export_mermaid_class_diagram(&graph, &export_opts_detailed);

    assert!(
        mermaid_detailed.contains("class InvoiceError"),
        "Detailed diagram must retain technical error enum InvoiceError:\n{mermaid_detailed}"
    );
    assert!(
        mermaid_detailed.contains("class InvoiceOptions"),
        "Detailed diagram must retain technical CLI options InvoiceOptions:\n{mermaid_detailed}"
    );
}

#[test]
fn test_ast_scans_trait_implementation_realization_and_strong_enums() {
    let ws = TempWorkspace::new("trait_realization_test");
    let ws_path = ws.path();

    fs::write(
        ws_path.join("Cargo.toml"),
        r#"[package]
name = "gateway-core"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let models_dir = ws_path.join("src").join("features").join("gateway");
    fs::create_dir_all(&models_dir).unwrap();

    let models_rs = models_dir.join("models.rs");
    fs::write(
        &models_rs,
        r#"//! Payment gateway domain models and contracts.

pub enum TransactionStatus {
    Pending,
    Settled,
}

pub struct Transaction {
    pub id: String,
    pub status: TransactionStatus,
}

pub trait PaymentGateway {
    fn process(&self, tx: &Transaction) -> bool;
}

pub struct StripeGateway {
    pub api_key: String,
}

impl PaymentGateway for StripeGateway {
    fn process(&self, tx: &Transaction) -> bool {
        true
    }
}
"#,
    )
    .unwrap();

    let topology_opts = TopologyOptions {
        workspace_root: ws_path.to_path_buf(),
        detect_symbols: true,
        ..Default::default()
    };

    let graph = scan_workspace_topology(&topology_opts).unwrap();

    let export_opts = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };
    let mermaid = export_mermaid_class_diagram(&graph, &export_opts);

    // 1. Verify Realization relationship: StripeGateway ..|> PaymentGateway
    assert!(
        mermaid.contains("StripeGateway ..|> PaymentGateway"),
        "Conceptual diagram must contain realization relationship 'StripeGateway ..|> PaymentGateway':\n{mermaid}"
    );

    // 2. Verify Composition: Transaction *-- TransactionStatus : status
    assert!(
        mermaid.contains("Transaction *-- TransactionStatus : status"),
        "Conceptual diagram must contain composition relationship:\n{mermaid}"
    );
}
