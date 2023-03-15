use std::collections::HashMap;
use std::str::FromStr;
use std::vec::IntoIter;

use http::header::HeaderName;
use http::header::InvalidHeaderName;
use http::header::InvalidHeaderValue;
use http::HeaderMap;
use http::HeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error(transparent)]
    HeaderFailure(#[from] InvalidHeaderName),

    #[error(transparent)]
    ValueFailure(#[from] InvalidHeaderValue),
}

#[derive(Default, Debug)]
pub struct Headers(HashMap<String, Vec<String>>);

impl Headers {
    /// Creates a new headers structure
    /// with the default fields.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let headers = Headers::new();
    ///
    /// assert_eq!(headers.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of headers
    /// defined in the current headers.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::new();
    ///
    /// assert_eq!(headers.len(), 0);
    ///
    /// headers.insert("Content-Type", "application/json");
    ///
    /// assert_eq!(headers.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of values
    /// defined for the given header.
    /// If the header is not defined,
    /// then 0 is returned.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::new();
    ///
    /// assert_eq!(headers.len_of("Content-Type"), 0);
    ///
    /// headers.append("Content-Type", "application/json");
    ///
    /// assert_eq!(headers.len_of("Content-Type"), 1);
    ///
    /// headers.append("Content-Type", "text/html");
    ///
    /// assert_eq!(headers.len_of("Content-Type"), 2);
    /// ```
    pub fn len_of(&self, header: &str) -> usize {
        match self.get(header) {
            Some(values) => values.len(),
            None => 0,
        }
    }

    /// Determines if the given header
    /// is defined in the current headers.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert!(headers.has("Content-Type"));
    /// ```
    pub fn has(&self, header: &str) -> bool {
        self.0.contains_key(&header.to_lowercase())
    }

    /// Only checks the first value of the header.
    /// The rest of the values (if any) will be ignored.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert!(headers.is("Content-Type", "application/json"));
    /// ```
    pub fn is(&self, header: &str, value: &str) -> bool {
        match self.first(header) {
            Some(current) => current == value,
            None => false,
        }
    }

    /// Only checks the first value of the header.
    /// The rest of the values (if any) will be ignored.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers =
    ///     Headers::from([("Content-Type", "application/json; charset=utf-8")]);
    ///
    /// assert!(headers.contains("Content-Type", "application/json"));
    /// ```
    pub fn contains(&self, header: &str, value: &str) -> bool {
        match self.first(header) {
            Some(header) => header.contains(value),
            None => false,
        }
    }

    /// Gets all the values associated with
    /// a header name.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["application/json".to_string()])
    /// );
    /// ```
    pub fn get(&self, header: &str) -> Option<&Vec<String>> {
        self.0.get(&header.to_lowercase())
    }

    pub fn get_mut(&mut self, header: &str) -> Option<&mut Vec<String>> {
        self.0.get_mut(&header.to_lowercase())
    }

    /// Gets the first value associated
    /// with a header name.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert_eq!(headers.first("Content-Type"), Some("application/json"));
    /// ```
    pub fn first(&self, header: &str) -> Option<&str> {
        self.get(header)?.first().map(|value| value.as_str())
    }

    /// Inserts (replaces) a given header with the given
    /// value. Removes any previous values for that header.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.insert("Content-Type", "text/plain");
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["text/plain".to_string()])
    /// );
    /// ```
    pub fn insert<H, T>(&mut self, header: H, value: T) -> &mut Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();
        self.0.insert(header.to_lowercase(), vec![value.into()]);

        self
    }

    /// Appends a value to the given header.
    /// If the header does not exist, it will be created.
    /// If the header already exists, the value will be
    /// appended.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.append("Content-Type", "text/plain");
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec![
    ///         "application/json".to_string(),
    ///         "text/plain".to_string()
    ///     ])
    /// );
    /// ```
    pub fn append<H, T>(&mut self, header: H, value: T) -> &mut Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();

        if let Some(values) = self.get_mut(&header) {
            values.push(value.into());

            return self;
        }

        self.insert(header, value)
    }

    /// Appends many values to the given header.
    /// If the header does not exist, it will be created.
    /// If the header already exists, the values will be
    /// appended.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.append_many("Content-Type", vec!["text/plain", "text/html"]);
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec![
    ///         "application/json".to_string(),
    ///         "text/plain".to_string(),
    ///         "text/html".to_string()
    ///     ])
    /// );
    /// ```
    pub fn append_many<H, T>(&mut self, header: H, values: Vec<T>) -> &mut Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();
        if let Some(current) = self.get_mut(&header) {
            let values = values.into_iter().map(|value| value.into());
            current.extend(values);

            return self;
        }

        self.replace(header, values)
    }

    /// Replaces a given header with the given
    /// values. Removes any previous values for that header.
    /// If the header does not exist, it will be created.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.replace("Content-Type", vec!["text/plain", "text/html"]);
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["text/plain".to_string(), "text/html".to_string()])
    /// );
    /// ```
    pub fn replace<H, T>(&mut self, header: H, values: Vec<T>) -> &mut Self
    where
        H: Into<String>,
        T: Into<String>,
    {
        let header: String = header.into();
        self.0.insert(
            header.to_lowercase(),
            values.into_iter().map(|value| value.into()).collect(),
        );

        self
    }

    /// Removes a given header.
    /// If the header does not exist, nothing will happen.
    /// If the header exists, all its values will be
    /// removed.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.remove("Content-Type");
    ///
    /// assert_eq!(headers.get("Content-Type"), None);
    /// ```
    pub fn remove<H>(&mut self, header: H) -> &mut Self
    where
        H: Into<String>,
    {
        let header: String = header.into();
        self.0.remove(&header.to_lowercase());

        self
    }

    /// Clears all the headers.
    /// After this operation, the headers will be empty.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    ///
    /// let mut headers = Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.clear();
    ///
    /// assert_eq!(headers.len(), 0);
    /// ```
    pub fn clear(&mut self) -> &mut Self {
        self.0.clear();

        self
    }
}

impl IntoIterator for Headers {
    type IntoIter = IntoIter<Self::Item>;
    type Item = (String, String);

    /// Transformation into an iterator over the headers.
    fn into_iter(self) -> Self::IntoIter {
        let mut result: Vec<(String, String)> = Vec::new();

        for (header, values) in self.0 {
            for value in values {
                result.push((header.clone(), value));
            }
        }

        result.into_iter()
    }
}

impl Extend<(String, String)> for Headers {
    /// Extends the headers with the given iterator.
    fn extend<T: IntoIterator<Item = (String, String)>>(&mut self, iter: T) {
        for (header, value) in iter {
            self.append(header, value);
        }
    }
}

impl<H, V> FromIterator<(H, V)> for Headers
where
    H: Into<String>,
    V: Into<String>,
{
    /// Creates a new `Headers` from an iterator.
    fn from_iter<T: IntoIterator<Item = (H, V)>>(iter: T) -> Self {
        let mut headers = Self::new();

        for (header, value) in iter {
            headers.insert(header, value);
        }

        headers
    }
}

impl<H, V, const N: usize> From<[(H, V); N]> for Headers
where
    H: Into<String>,
    V: Into<String>,
{
    /// Creates a new `Headers` from an array.
    fn from(values: [(H, V); N]) -> Self {
        Self::from_iter(values)
    }
}

impl TryFrom<Headers> for HeaderMap {
    type Error = ConversionError;

    /// Converts the `Headers` into a `HeaderMap`.
    fn try_from(from: Headers) -> Result<Self, Self::Error> {
        let mut headers = Self::new();

        for (header, value) in from {
            let header = HeaderName::from_str(&header)?;
            let value = HeaderValue::from_str(&value)?;

            headers.append(header.clone(), value);
        }

        Ok(headers)
    }
}

pub trait HasHeaders {
    fn headers(&self) -> &Headers;
    fn headers_mut(&mut self) -> &mut Headers;
}
