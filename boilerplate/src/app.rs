use std::sync::Arc;
use std::time::Duration;

use valar::database::Database;
// use valar::http::session::Session;
use valar::services::cache::MemoryCache;
use valar::services::Cacheable;
// use valar::services::Service;
// use valar::services::Singleton;

pub struct App {
    // pub database: Database,
    // pub cache: Arc<Cacheable>,
}

// impl Singleton<Cacheable> for App {
//     fn singleton(&self) -> Arc<Cacheable> {
//         self.cache.clone()
//     }
// }

impl App {
    // fn cache() -> Arc<Cacheable> {
    //     Arc::new(MemoryCache::new(Duration::from_secs(1)))
    // }

    pub async fn create() -> Self {
        // let database = Database::connect("host=localhost
        // user=erik dbname=valar")     .await
        //     .expect("Unable to connect to the database");

        Self {
            // database,
            // cache: Self::cache(),
        }
    }

    pub async fn fake() -> Self {
        Self::create().await
    }
}
