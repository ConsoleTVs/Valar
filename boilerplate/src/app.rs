use std::time::Duration;

use valar::database::Database;
use valar::services::cache::MemoryCache;
use valar::services::Cache;
use valar::Application;

pub struct App {
    pub database: Database,
    pub cache: Box<dyn Cache + Send + Sync>,
}

impl Application for App {}

impl App {
    fn cache() -> impl Cache + Send + Sync {
        MemoryCache::with_purge_interval(Duration::from_secs(1))
    }

    pub async fn create() -> Self {
        let database = Database::connect("host=localhost user=erik dbname=valar")
            .await
            .expect("Unable to connect to the database");

        Self {
            database,
            cache: Box::new(Self::cache()),
        }
    }

    pub async fn fake() -> Self {
        Self::create().await
    }
}
