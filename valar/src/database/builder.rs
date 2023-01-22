pub mod parameters;
pub mod selects;
pub mod wheres;

pub use parameters::Parameters;
pub use selects::SelectQueryBuilder;
pub use wheres::Operation;
pub use wheres::Where;
pub use wheres::Whereable;

pub struct QueryBuilder {
    table: String,
}

impl QueryBuilder {
    pub fn table<T>(table: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            table: table.into(),
        }
    }

    #[must_use]
    pub fn select<'a, T, C>(self, columns: C) -> SelectQueryBuilder<'a>
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        SelectQueryBuilder::new(self.table, columns)
    }

    #[must_use]
    pub fn select_all<'a>(self) -> SelectQueryBuilder<'a> {
        SelectQueryBuilder::new(self.table, ["*"])
    }
}

pub trait ToSqlString<'a> {
    fn to_sql_string(&self, parameters: &mut Parameters<'a>) -> String;
}
