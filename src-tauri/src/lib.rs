mod cache;
mod commands;
mod db;
mod error;
mod ingest;
mod state;

use std::sync::Mutex;

use cache::CacheManager;
use directories::ProjectDirs;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cache_path = ProjectDirs::from("com", "autorag", "data")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("autorag-data-cache"));

    let app_state = AppState::new(cache_path.clone());

    let cache_manager = CacheManager::new(cache_path).ok();

    tauri::Builder::default()
        .manage(app_state)
        .manage(Mutex::new(cache_manager))
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                let window = tauri::Manager::get_webview_window(_app, "main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Database commands
            commands::connect_database,
            commands::disconnect_database,
            commands::test_connection,
            commands::is_connected,
            // Document commands
            commands::list_files,
            commands::list_files_with_documents,
            commands::list_documents,
            commands::get_document,
            commands::get_document_with_pages,
            commands::get_pages,
            commands::get_page_chunks,
            commands::get_file_path,
            commands::get_document_page_count,
            // Cache commands
            commands::get_thumbnail_url,
            commands::get_preview_url,
            commands::get_page_image_url,
            commands::get_chunk_image_url,
            commands::clear_cache,
            commands::clear_db_cache,
            commands::get_cache_size,
            commands::prefetch_document_thumbnails,
            // Query commands
            commands::create_query,
            commands::update_query,
            commands::delete_query,
            commands::list_queries,
            commands::get_query_with_evidence,
            commands::add_retrieval_relation,
            commands::remove_retrieval_relation,
            commands::remove_evidence_group,
            commands::reorder_evidence,
            commands::update_retrieval_score,
            // Ingest commands
            commands::ingest_pdf,
            commands::ingest_images,
            commands::get_supported_formats,
            // Export commands
            commands::get_export_counts,
            commands::export_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
