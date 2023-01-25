pub mod builder;
pub mod executor;
pub mod query;

pub use tokio_postgres::types::ToSql;
pub use tokio_postgres::Client;
pub use tokio_postgres::Config;
pub use tokio_postgres::Error as PGError;
pub use tokio_postgres::Row;

pub use crate::database::builder::QueryBuilder;
pub use crate::database::executor::Executor;
pub use crate::database::query::PendingQuery;

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Database, PGError> {
        let (client, connection) =
            tokio_postgres::connect(url, tokio_postgres::NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Database { client })
    }

    pub async fn connect_with(config: Config) -> Result<Database, PGError> {
        let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Database { client })
    }

    #[must_use]
    pub fn query<'a, T>(statement: T) -> PendingQuery<'a>
    where
        T: Into<String>,
    {
        PendingQuery::new(statement)
    }

    pub fn table<T>(table: T) -> QueryBuilder
    where
        T: Into<String>,
    {
        QueryBuilder::table(table)
    }
}

pub trait ToPendingQuery {
    fn to_pending_query(&self) -> PendingQuery;
}
