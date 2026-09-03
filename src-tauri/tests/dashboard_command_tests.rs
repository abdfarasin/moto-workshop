use serde_json::json;
use tempfile::tempdir;

use moto_workshop_lib::{
    commands::{
        dashboard::{handle_load_dashboard, LoadDashboardCommandInput},
        service_visit_workspace::CommandErrorCategory,
    },
    runtime::database::RuntimeDatabase,
};

#[test]
fn dashboard_command_returns_the_exact_camel_case_read_model() {
    // # Arrange
    let directory = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(directory.path()).unwrap();
    let input: LoadDashboardCommandInput = serde_json::from_value(json!({
        "dayStartMs": 10_000,
        "dayEndMs": 20_000
    }))
    .unwrap();

    // # Act
    let dashboard = handle_load_dashboard(&database, input).unwrap();
    let serialized = serde_json::to_value(dashboard).unwrap();

    // # Assert
    assert_eq!(serialized["summary"]["activeServiceVisits"], 0);
    assert_eq!(serialized["summary"]["issuedInvoiceValueTodayFils"], 0);
    assert_eq!(serialized["recentServiceVisits"], json!([]));
    assert_eq!(serialized["recentInvoices"], json!([]));
    assert_eq!(serialized["inventoryAlerts"], json!([]));
}

#[test]
fn dashboard_command_rejects_unknown_fields_and_sanitizes_invalid_ranges() {
    // # Arrange
    let unsafe_input = json!({ "dayStartMs": 1, "dayEndMs": 2, "databasePath": "forged.db" });
    let directory = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(directory.path()).unwrap();

    // # Act
    let parsed = serde_json::from_value::<LoadDashboardCommandInput>(unsafe_input);
    let error = handle_load_dashboard(
        &database,
        LoadDashboardCommandInput {
            day_start_ms: 20_000,
            day_end_ms: 10_000,
        },
    )
    .unwrap_err();

    // # Assert
    assert!(parsed.is_err());
    assert_eq!(error.category, CommandErrorCategory::ValidationError);
    assert!(!error.message.to_ascii_lowercase().contains("select"));
}
