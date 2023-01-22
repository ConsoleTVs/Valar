use async_trait::async_trait;
use valar::database::Database;
use valar::Application;
use valar::Error;

pub struct App {
    pub database: Database,
}

#[async_trait]
impl Application for App {
    async fn create() -> Result<Self, Error> {
        let app = Self {
            database: Database::connect("host=localhost user=erik dbname=valar").await?,
        };

        Ok(app)
    }
}
