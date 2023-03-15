use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use valar::database::Database;
use valar::drivers::cache::MemoryCache;
use valar::Application;

pub struct App {
    pub database: Database,
    pub cache: MemoryCache,
}
#[async_trait]
impl Application for App {}

impl App {
    pub async fn create() -> Arc<Self> {
        let database = Database::connect("host=localhost user=erik dbname=valar")
            .await
            .expect("Unable to connect to the database");

        let cache = MemoryCache::with_purge_interval(Duration::from_secs(1));

        let app = Self { database, cache };

        Arc::new(app)
    }

    pub async fn fake() -> Arc<Self> {
        Self::create().await
    }
}
