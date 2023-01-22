use crate::database::builder::Parameters;
use crate::database::builder::ToSqlString;
use tokio_postgres::types::ToSql;

pub enum Operation<'a> {
    Equal(String, &'a (dyn ToSql + Sync)),
    NotEqual(String, &'a (dyn ToSql + Sync)),
    GreaterThan(String, &'a (dyn ToSql + Sync)),
    GreaterThanOrEqual(String, &'a (dyn ToSql + Sync)),
    LessThan(String, &'a (dyn ToSql + Sync)),
    LessThanOrEqual(String, &'a (dyn ToSql + Sync)),
    Like(String, &'a (dyn ToSql + Sync)),
    NotLike(String, &'a (dyn ToSql + Sync)),
    In(String, Vec<&'a (dyn ToSql + Sync)>),
    NotIn(String, Vec<&'a (dyn ToSql + Sync)>),
    Between(String, &'a (dyn ToSql + Sync), &'a (dyn ToSql + Sync)),
    NotBetween(String, &'a (dyn ToSql + Sync), &'a (dyn ToSql + Sync)),
    IsNull(String),
    IsNotNull(String),
}

impl<'a> ToSqlString<'a> for Operation<'a> {
    fn to_sql_string(&self, parameters: &mut Parameters<'a>) -> String {
        match self {
            Self::Equal(column, value) => {
                format!("{column} = ${}", parameters.add(*value))
            }
            Self::NotEqual(column, value) => {
                format!("{column} != ${}", parameters.add(*value))
            }
            Self::GreaterThan(column, value) => {
                format!("{column} > ${}", parameters.add(*value))
            }
            Self::GreaterThanOrEqual(column, value) => {
                format!("{column} >= ${}", parameters.add(*value))
            }
            Self::LessThan(column, value) => {
                format!("{column} < ${}", parameters.add(*value))
            }
            Self::LessThanOrEqual(column, value) => {
                format!("{column} <= ${}", parameters.add(*value))
            }
            Self::Like(column, value) => {
                format!("{column} LIKE ${}", parameters.add(*value))
            }
            Self::NotLike(column, value) => {
                format!("{column} NOT LIKE ${}", parameters.add(*value))
            }
            Self::In(column, values) => {
                let positions = values
                    .iter()
                    .map(|value| format!("${}", parameters.add(*value)))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{column} IN ({positions})")
            }
            Self::NotIn(column, values) => {
                let positions = values
                    .iter()
                    .map(|value| format!("${}", parameters.add(*value)))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{column} NOT IN ({positions})")
            }
            Self::Between(column, min, max) => {
                let min_position = parameters.add(*min);
                let max_position = parameters.add(*max);

                format!("{column} BETWEEN ${min_position} AND ${max_position}")
            }
            Self::NotBetween(column, min, max) => {
                let min_position = parameters.add(*min);
                let max_position = parameters.add(*max);

                format!("{column} NOT BETWEEN ${min_position} AND ${max_position}")
            }
            Self::IsNull(column) => {
                format!("{column} IS NULL")
            }
            Self::IsNotNull(column) => {
                format!("{column} IS NOT NULL")
            }
        }
    }
}

pub enum Where<'a> {
    And(Operation<'a>),
    Or(Operation<'a>),
    Nop(Operation<'a>),
}

impl<'a> Where<'a> {
    pub fn into_nop(self) -> Self {
        match self {
            Self::And(operation) => Self::Nop(operation),
            Self::Or(operation) => Self::Nop(operation),
            Self::Nop(operation) => Self::Nop(operation),
        }
    }
}

impl<'a> ToSqlString<'a> for Where<'a> {
    fn to_sql_string(&self, parameters: &mut Parameters<'a>) -> String {
        match self {
            Self::And(operation) => format!("AND ({})", operation.to_sql_string(parameters)),
            Self::Or(operation) => format!("OR ({})", operation.to_sql_string(parameters)),
            Self::Nop(operation) => format!("({})", operation.to_sql_string(parameters)),
        }
    }
}

pub trait Whereable<'a>: Sized {
    fn add_where(&mut self, condition: Where<'a>);

    fn where_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::Equal(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_equal<C, V>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::Equal(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_not_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::NotEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_not_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::NotEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_greater_than<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::GreaterThan(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_greater_than<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::GreaterThan(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_greater_than_or_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::GreaterThanOrEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_greater_than_or_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::GreaterThanOrEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_less_than<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::LessThan(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_less_than<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::LessThan(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_less_than_or_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::LessThanOrEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_less_than_or_equal<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::LessThanOrEqual(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_like<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::Like(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_like<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::Like(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_not_like<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::NotLike(column.into(), value));
        self.add_where(condition);

        self
    }

    fn or_where_not_like<C>(mut self, column: C, value: &'a (dyn ToSql + Sync)) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::NotLike(column.into(), value));
        self.add_where(condition);

        self
    }

    fn where_in<C, V>(mut self, column: C, values: V) -> Self
    where
        C: Into<String>,
        V: Into<Vec<&'a (dyn ToSql + Sync)>>,
    {
        let condition = Where::And(Operation::In(column.into(), values.into()));
        self.add_where(condition);

        self
    }

    fn or_where_in<C, V>(mut self, column: C, values: V) -> Self
    where
        C: Into<String>,
        V: Into<Vec<&'a (dyn ToSql + Sync)>>,
    {
        let condition = Where::Or(Operation::In(column.into(), values.into()));
        self.add_where(condition);

        self
    }

    fn where_not_in<C, V>(mut self, column: C, values: V) -> Self
    where
        C: Into<String>,
        V: Into<Vec<&'a (dyn ToSql + Sync)>>,
    {
        let condition = Where::And(Operation::NotIn(column.into(), values.into()));
        self.add_where(condition);

        self
    }

    fn or_where_not_in<C, V>(mut self, column: C, values: V) -> Self
    where
        C: Into<String>,
        V: Into<Vec<&'a (dyn ToSql + Sync)>>,
    {
        let condition = Where::Or(Operation::NotIn(column.into(), values.into()));
        self.add_where(condition);

        self
    }

    fn where_null<C>(mut self, column: C) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::IsNull(column.into()));
        self.add_where(condition);

        self
    }

    fn or_where_null<C>(mut self, column: C) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::IsNull(column.into()));
        self.add_where(condition);

        self
    }

    fn where_not_null<C>(mut self, column: C) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::IsNotNull(column.into()));
        self.add_where(condition);

        self
    }

    fn or_where_not_null<C>(mut self, column: C) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::IsNotNull(column.into()));
        self.add_where(condition);

        self
    }

    fn where_between<C>(
        mut self,
        column: C,
        start: &'a (dyn ToSql + Sync),
        end: &'a (dyn ToSql + Sync),
    ) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::Between(column.into(), start, end));
        self.add_where(condition);

        self
    }

    fn or_where_between<C>(
        mut self,
        column: C,
        start: &'a (dyn ToSql + Sync),
        end: &'a (dyn ToSql + Sync),
    ) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::Between(column.into(), start, end));
        self.add_where(condition);

        self
    }

    fn where_not_between<C>(
        mut self,
        column: C,
        start: &'a (dyn ToSql + Sync),
        end: &'a (dyn ToSql + Sync),
    ) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::And(Operation::NotBetween(column.into(), start, end));
        self.add_where(condition);

        self
    }

    fn or_where_not_between<C>(
        mut self,
        column: C,
        start: &'a (dyn ToSql + Sync),
        end: &'a (dyn ToSql + Sync),
    ) -> Self
    where
        C: Into<String>,
    {
        let condition = Where::Or(Operation::NotBetween(column.into(), start, end));
        self.add_where(condition);

        self
    }
}
