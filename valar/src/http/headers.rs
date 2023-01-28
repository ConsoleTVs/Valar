use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::str::FromStr;

use http::header::HeaderName;
use http::header::InvalidHeaderName;
use http::header::InvalidHeaderValue;
use http::HeaderMap;
use http::HeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("{}", .0)]
    HeaderFailure(InvalidHeaderName),
    #[error("{}", .0)]
    ValueFailure(InvalidHeaderValue),
}

impl From<InvalidHeaderName> for ConversionError {
    fn from(error: InvalidHeaderName) -> Self {
        ConversionError::HeaderFailure(error)
    }
}

impl From<InvalidHeaderValue> for ConversionError {
    fn from(error: InvalidHeaderValue) -> Self {
        ConversionError::ValueFailure(error)
    }
}

#[derive(Default, Debug)]
pub struct Headers(HashMap<String, Vec<String>>);

impl Headers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, header: &str) -> bool {
        self.0.contains_key(header)
    }

    /// Only checks the first value of the header.
    /// The rest of the values (if any) will be ignored.
    pub fn is(&self, header: &str, value: &str) -> bool {
        self.first(header).map_or(false, |current| current == value)
    }

    /// Only checks the first value of the header.
    /// The rest of the values (if any) will be ignored.
    pub fn contains(&self, header: &str, value: &str) -> bool {
        self.first(header)
            .map_or(false, |current| current.contains(value))
    }

    pub fn get(&self, header: &str) -> Option<&Vec<String>> {
        self.0.get(header)
    }

    pub fn first(&self, header: &str) -> Option<&String> {
        self.get(header).and_then(|header| header.first())
    }

    pub fn insert<H, T>(&mut self, header: H, value: T) -> &Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        self.0.insert(header.into(), vec![value.into()]);

        self
    }

    pub fn append<H, T>(&mut self, header: H, value: T) -> &Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();

        if let Some(values) = self.0.get_mut(&header) {
            values.push(value.into());

            return self;
        }

        self.insert(header, value)
    }

    pub fn append_many<H, T>(&mut self, header: H, values: Vec<T>) -> &Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();
        if let Some(current) = self.0.get_mut(&header) {
            let values = values.into_iter().map(|value| value.into());
            current.extend(values);

            return self;
        }

        self.replace(header, values)
    }

    pub fn replace<H, T>(&mut self, header: H, values: Vec<T>) -> &Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        self.0.insert(
            header.into(),
            values.into_iter().map(|value| value.into()).collect(),
        );

        self
    }
}

impl IntoIterator for Headers {
    type IntoIter = IntoIter<String, Vec<String>>;
    type Item = (String, Vec<String>);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<(String, Vec<String>)> for Headers {
    fn extend<T: IntoIterator<Item = (String, Vec<String>)>>(&mut self, iter: T) {
        for (header, values) in iter {
            self.append_many(header, values);
        }
    }
}

impl<H, V> FromIterator<(H, V)> for Headers
where
    H: Into<String>,
    V: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (H, V)>>(iter: T) -> Self {
        let mut headers = Self::new();

        for (header, value) in iter {
            headers.insert(header, value);
        }

        headers
    }
}

impl<'a, const N: usize> From<[(&'a str, &'a str); N]> for Headers {
    fn from(values: [(&'a str, &'a str); N]) -> Self {
        Self::from_iter(values)
    }
}

impl TryFrom<Headers> for HeaderMap {
    type Error = ConversionError;

    fn try_from(from: Headers) -> Result<Self, Self::Error> {
        let mut headers = Self::new();

        for (header, values) in from {
            let header = HeaderName::from_str(&header)?;
            for value in values {
                let value = HeaderValue::from_str(&value)?;
                headers.append(header.clone(), value);
            }
        }

        Ok(headers)
    }
}
