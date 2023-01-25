use tokio_postgres::types::ToSql;

use crate::database::Executor;

pub struct PendingQuery<'a> {
    statement: String,
    parameters: Vec<&'a (dyn ToSql + Sync)>,
}

impl<'a> PendingQuery<'a> {
    pub fn new<T>(statement: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            statement: statement.into(),
            parameters: vec![],
        }
    }

    #[must_use]
    pub fn parameters_from<V>(mut self, value: V) -> Self
    where
        V: Into<Vec<&'a (dyn ToSql + Sync)>>,
    {
        self.parameters = value.into();

        self
    }

    #[must_use]
    pub fn parameters<const N: usize>(
        mut self,
        value: [&'a (dyn ToSql + Sync); N],
    ) -> Self {
        self.parameters = Vec::from(value);

        self
    }

    #[must_use]
    pub fn with<T: ToSql + Sync>(mut self, value: &'a T) -> Self {
        self.parameters.push(value);

        self
    }
}

impl<'a> Executor<'a> for PendingQuery<'a> {
    fn executor_parameters(&self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        (self.statement.clone(), self.parameters.clone())
    }
}

impl<'a> ToString for PendingQuery<'a> {
    fn to_string(&self) -> String {
        self.statement.clone()
    }
}
