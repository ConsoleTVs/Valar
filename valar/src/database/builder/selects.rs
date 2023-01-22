use crate::database::builder::Parameters;
use crate::database::builder::ToSqlString;
use crate::database::builder::Where;
use crate::database::builder::Whereable;
use crate::database::Executor;
use crate::database::PendingQuery;
use crate::database::ToPendingQuery;
use tokio_postgres::types::ToSql;

pub struct SelectQueryBuilder<'a> {
    table: String,
    columns: Vec<String>,
    wheres: Vec<Where<'a>>,
}

impl<'a> Whereable<'a> for SelectQueryBuilder<'a> {
    fn add_where(&mut self, condition: Where<'a>) {
        if self.wheres.is_empty() {
            return self.wheres.push(condition.into_nop());
        }

        self.wheres.push(condition)
    }
}

impl<'a> SelectQueryBuilder<'a> {
    #[must_use]
    pub fn new<T, C>(table: String, columns: C) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            table,
            columns: columns.into_iter().map(|column| column.into()).collect(),
            wheres: vec![],
        }
    }
}

impl<'a> ToPendingQuery for SelectQueryBuilder<'a> {
    fn to_pending_query(&self) -> PendingQuery {
        let mut parameters = Parameters::new();
        let columns = self.columns.join(", ");
        let table = &self.table;
        let mut statement = format!("SELECT {columns} FROM {table}");

        if !self.wheres.is_empty() {
            let wheres: Vec<String> = self
                .wheres
                .iter()
                .map(|condition| condition.to_sql_string(&mut parameters))
                .collect();
            let wheres = wheres.join(" ");

            statement.push_str(&format!(" WHERE ({})", wheres));
        }

        PendingQuery::new(statement).parameters_from(parameters)
    }
}

impl<'a> Executor<'a> for SelectQueryBuilder<'a> {
    fn executor_parameters(&self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut parameters = Parameters::new();
        let columns = self.columns.join(", ");
        let table = &self.table;
        let mut statement = format!("SELECT {columns} FROM {table}");

        if !self.wheres.is_empty() {
            let wheres: Vec<String> = self
                .wheres
                .iter()
                .map(|condition| condition.to_sql_string(&mut parameters))
                .collect();
            let wheres = wheres.join(" ");

            statement.push_str(&format!(" WHERE ({})", wheres));
        }

        (statement, parameters.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::builder::wheres::Whereable;
    use crate::database::builder::QueryBuilder;
    use crate::database::ToPendingQuery;

    #[test]
    fn test_simple_select() {
        let query = QueryBuilder::table("users")
            .select(["id"])
            .to_pending_query()
            .to_string();

        assert_eq!(query, "SELECT id FROM users");
    }

    #[test]
    fn test_where_select() {
        let query = QueryBuilder::table("users")
            .select(["id"])
            .where_equal("name", &"Erik")
            .to_pending_query()
            .to_string();

        assert_eq!(query, "SELECT id FROM users WHERE ((name = $1))");
    }

    #[test]
    fn test_where_select_2() {
        let query = QueryBuilder::table("users")
            .select(["id"])
            .where_equal("name", &"Erik")
            .where_equal("email", &"soc@erik.cat")
            .to_pending_query()
            .to_string();

        assert_eq!(
            query,
            "SELECT id FROM users WHERE ((name = $1) AND (email = $2))"
        );
    }

    #[test]
    fn test_where_select_3() {
        let query = QueryBuilder::table("users")
            .select(["id"])
            .where_equal("name", &"Erik")
            .where_not_equal("email", &"soc@erik.cat")
            .where_between("age", &"18", &"30")
            .to_pending_query()
            .to_string();

        assert_eq!(
            query,
            "SELECT id FROM users WHERE ((name = $1) AND (email != $2) AND (age BETWEEN $3 AND $4))"
        );
    }
}
