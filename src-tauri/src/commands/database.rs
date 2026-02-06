use tauri::State;

use crate::db::{create_pool, test_connection as test_db_connection, DatabaseConfig};
use crate::error::{AppError, Result};
use crate::state::AppState;

#[tauri::command]
pub async fn connect_database(config: DatabaseConfig, state: State<'_, AppState>) -> Result<bool> {
    let pool = create_pool(&config).await?;
    test_db_connection(&pool).await?;
    state.set_connection(pool).await;
    Ok(true)
}

#[tauri::command]
pub async fn disconnect_database(state: State<'_, AppState>) -> Result<bool> {
    state.clear_connection().await;
    Ok(true)
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;
    test_db_connection(&pool).await
}

#[tauri::command]
pub async fn is_connected(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.is_connected().await)
}
