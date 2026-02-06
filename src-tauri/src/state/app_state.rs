use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub pool: Arc<RwLock<Option<PgPool>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_connection(&self, pool: PgPool) {
        let mut pool_guard = self.pool.write().await;
        *pool_guard = Some(pool);
    }

    pub async fn clear_connection(&self) {
        let mut pool_guard = self.pool.write().await;
        if let Some(pool) = pool_guard.take() {
            pool.close().await;
        }
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
