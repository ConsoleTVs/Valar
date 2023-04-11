use std::sync::Arc;
use std::time::Duration;

use valar::database::Database;
use valar::services::cache::MemoryCache;
use valar::services::Cacheable;
use valar::Application;

pub struct App {
    pub database: Database,
    pub cache: Arc<Cacheable>,
}

impl Application for App {}

impl App {
    fn cache() -> Arc<Cacheable> {
        Arc::new(MemoryCache::new(Duration::from_secs(1)))
    }

    pub async fn create() -> Self {
        let database = Database::connect("host=localhost user=erik dbname=valar")
            .await
            .expect("Unable to connect to the database");

        Self {
            database,
            cache: Self::cache(),
        }
    }

    pub async fn fake() -> Self {
        Self::create().await
    }
}
