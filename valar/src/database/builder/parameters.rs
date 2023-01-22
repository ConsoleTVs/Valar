use tokio_postgres::types::ToSql;

#[derive(Default)]
pub struct Parameters<'a> {
    values: Vec<&'a (dyn ToSql + Sync)>,
}

impl<'a> Parameters<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, value: &'a (dyn ToSql + Sync)) -> usize {
        self.values.push(value);

        self.values.len()
    }
}

impl<'a> From<Parameters<'a>> for Vec<&'a (dyn ToSql + Sync)> {
    fn from(parameters: Parameters<'a>) -> Self {
        parameters.values
    }
}
