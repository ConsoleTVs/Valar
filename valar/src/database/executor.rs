use async_trait::async_trait;
use tokio_postgres::types::ToSql;
use tokio_postgres::Error as PGError;
use tokio_postgres::Row;

use crate::database::Database;

#[async_trait]
pub trait Executor<'a> {
    fn executor_parameters(&self) -> (String, Vec<&'a (dyn ToSql + Sync)>);

    async fn raw_get(&self, database: &Database) -> Result<Vec<Row>, PGError> {
        let (statement, parameters) = &self.executor_parameters();

        database.client.query(statement, parameters).await
    }

    async fn raw_first(&self, database: &Database) -> Result<Row, PGError> {
        let (statement, parameters) = &self.executor_parameters();

        database.client.query_one(statement, parameters).await
    }

    async fn execute(&self, database: &Database) -> Result<u64, PGError> {
        let (statement, parameters) = &self.executor_parameters();

        database.client.execute(statement, parameters).await
    }

    async fn get<T, R>(&self, database: &Database) -> Result<R, PGError>
    where
        T: TryFrom<Row, Error = PGError>,
        R: FromIterator<T>,
    {
        self.raw_get(database)
            .await?
            .into_iter()
            .map(|row| T::try_from(row))
            .collect()
    }

    async fn first<T>(&self, database: &Database) -> Result<T, PGError>
    where
        T: TryFrom<Row, Error = PGError>,
    {
        T::try_from(self.raw_first(database).await?)
    }
}
