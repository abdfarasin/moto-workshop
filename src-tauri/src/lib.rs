pub mod application;
pub mod commands;
pub mod db;
pub mod domain;
mod repositories;
pub mod runtime;

use commands::customer::create_customer;
use commands::motorcycle_registration::load_motorcycle_registration_reference_data;
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
