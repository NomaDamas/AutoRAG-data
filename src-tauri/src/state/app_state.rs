use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub pool: Arc<RwLock<Option<PgPool>>>,
    pub cache_path: PathBuf,
    pub current_db_name: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            cache_path,
            current_db_name: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_db_identifier(&self) -> Option<String> {
        let guard = self.current_db_name.read().await;
        guard.clone()
    }

    pub async fn set_connection(&self, pool: PgPool, db_name: String) {
        let mut pool_guard = self.pool.write().await;
        *pool_guard = Some(pool);
        let mut db_guard = self.current_db_name.write().await;
        *db_guard = Some(db_name);
    }

    pub async fn clear_connection(&self) {
        let mut pool_guard = self.pool.write().await;
        if let Some(pool) = pool_guard.take() {
            pool.close().await;
        }
        let mut db_guard = self.current_db_name.write().await;
        *db_guard = None;
    }

    pub async fn get_pool(&self) -> Option<PgPool> {
        let guard = self.pool.read().await;
        guard.clone()
    }

    pub async fn is_connected(&self) -> bool {
        let guard = self.pool.read().await;
        guard.is_some()
    }
}
