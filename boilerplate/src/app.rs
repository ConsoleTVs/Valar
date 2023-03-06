use std::time::Duration;

use async_trait::async_trait;
use valar::database::Database;
use valar::drivers::cache::MemoryCache;
use valar::drivers::Cache;
use valar::Application;
use valar::Error;

pub struct App {
    pub database: Database,
    pub cache: MemoryCache,
}

#[async_trait]
impl Application for App {
    async fn create() -> Result<Self, Error> {
        let app = Self {
            database: Database::connect("host=localhost user=erik dbname=valar").await?,
            cache: MemoryCache::with_purge_interval(Duration::from_secs(1)),
        };

        app.cache
            .insert_for("foo".to_string(), "bar".to_string(), Duration::from_secs(3))
            .await?;

        Ok(app)
    }
}
