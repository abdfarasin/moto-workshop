pub mod application;
pub mod commands;
pub mod db;
pub mod domain;
mod repositories;
pub mod runtime;

use commands::customer::{create_customer, search_customer_directory};
use commands::customer_details::load_customer_details;
use commands::dashboard::load_dashboard;
use commands::inventory::{
    adjust_inventory_stock, create_inventory_item, list_inventory_units,
    load_inventory_item_details, search_inventory_items, update_inventory_item,
};
use commands::invoice::{
    issue_invoice, list_invoices, load_invoice_details, load_service_visit_invoice,
};
use commands::motorcycle_directory::{load_motorcycle_details, search_motorcycle_directory};
use commands::motorcycle_registration::{
    create_motorcycle, load_motorcycle_registration_reference_data,
};
use commands::service_visit_directory::list_service_visits;
use commands::service_visit_lookup::{list_customer_motorcycles, search_customers};
use commands::service_visit_workspace::{
    add_service_visit_part, cancel_service_visit, close_service_visit, create_service_visit,
    list_service_visit_inventory_items, load_service_visit_workspace,
    mark_service_visit_ready_for_pickup, reopen_service_visit, update_service_visit_work,
    void_service_visit_part,
};

use runtime::database::RuntimeDatabase;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|application| {
            let application_data_directory = application.path().app_data_dir()?;

            let database = RuntimeDatabase::initialize(application_data_directory)?;

            application.manage(database);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_customer,
            search_customer_directory,
            load_customer_details,
            load_dashboard,
            search_motorcycle_directory,
            load_motorcycle_details,
            search_inventory_items,
            load_inventory_item_details,
            list_inventory_units,
            create_inventory_item,
            update_inventory_item,
            adjust_inventory_stock,
            list_invoices,
            load_invoice_details,
            load_service_visit_invoice,
            issue_invoice,
            list_service_visits,
            create_motorcycle,
            load_motorcycle_registration_reference_data,
            search_customers,
            list_customer_motorcycles,
            create_service_visit,
            load_service_visit_workspace,
            list_service_visit_inventory_items,
            update_service_visit_work,
            add_service_visit_part,
            void_service_visit_part,
            mark_service_visit_ready_for_pickup,
            reopen_service_visit,
            close_service_visit,
            cancel_service_visit,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| panic!("Moto Workshop startup failed: {error}"));
}
