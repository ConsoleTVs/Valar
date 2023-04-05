use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;
use std::vec::IntoIter;

use http::header::HeaderName;
use http::header::InvalidHeaderName;
use http::header::InvalidHeaderValue;
use http::HeaderMap;
use http::HeaderValue;
use thiserror::Error;

use crate::http::Cookie;
use crate::http::Request;
use crate::http::Response;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error(transparent)]
    HeaderFailure(#[from] InvalidHeaderName),

    #[error(transparent)]
    ValueFailure(#[from] InvalidHeaderValue),
}

#[derive(Debug)]
pub struct Headers<T> {
    headers: HashMap<String, Vec<String>>,
    _marker: PhantomData<T>,
}

impl<T> Default for Headers<T> {
    fn default() -> Self {
        Self {
            headers: HashMap::new(),
            _marker: PhantomData::<T>,
        }
    }
}

impl<T> Headers<T> {
    /// Creates a new headers structure
    /// with the default fields.
    ///
    /// # Example
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let headers: Headers<Request> = Headers::new(HashMap::new());
    ///
    /// assert_eq!(headers.len(), 0);
    /// ```
    pub fn new(headers: HashMap<String, Vec<String>>) -> Self {
        Self {
            headers,
            ..Self::default()
        }
    }

    /// Returns the number of headers
    /// defined in the current headers.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> = Headers::default();
    ///
    /// assert_eq!(headers.len(), 0);
    ///
    /// headers.insert("Content-Type", "application/json");
    ///
    /// assert_eq!(headers.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Returns the number of values
    /// defined for the given header.
    /// If the header is not defined,
    /// then 0 is returned.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> = Headers::default();
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
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert!(headers.has("Content-Type"));
    /// ```
    pub fn has(&self, header: &str) -> bool {
        self.headers.contains_key(&header.to_lowercase())
    }

    /// Only checks the first value of the header.
    /// The rest of the values (if any) will be ignored.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
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
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
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
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["application/json".to_string()])
    /// );
    /// ```
    pub fn get(&self, header: &str) -> Option<&Vec<String>> {
        self.headers.get(&header.to_lowercase())
    }

    pub fn get_mut(&mut self, header: &str) -> Option<&mut Vec<String>> {
        self.headers.get_mut(&header.to_lowercase())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.headers
            .iter()
            .flat_map(|(header, values)| values.iter().map(move |value| (header, value)))
    }

    /// Gets the first value associated
    /// with a header name.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
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
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.insert("Content-Type", "text/plain");
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["text/plain".to_string()])
    /// );
    /// ```
    pub fn insert<H, V>(&mut self, header: H, value: V)
    where
        H: Into<String>,
        V: Into<String>,
    {
        let header: String = header.into();
        self.headers
            .insert(header.to_lowercase(), vec![value.into()]);
    }

    /// Replaces a given header with the given
    /// values. Removes any previous values for that header.
    /// If the header does not exist, it will be created.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.insert_many("Content-Type", vec!["text/plain", "text/html"]);
    ///
    /// assert_eq!(
    ///     headers.get("Content-Type"),
    ///     Some(&vec!["text/plain".to_string(), "text/html".to_string()])
    /// );
    /// ```
    pub fn insert_many<H, V>(&mut self, header: H, values: Vec<V>)
    where
        H: Into<String>,
        V: Into<String>,
    {
        let header: String = header.into();
        self.headers.insert(
            header.to_lowercase(),
            values.into_iter().map(|value| value.into()).collect(),
        );
    }

    /// Appends a value to the given header.
    /// If the header does not exist, it will be created.
    /// If the header already exists, the value will be
    /// appended.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
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
    pub fn append<H, V>(&mut self, header: H, value: V)
    where
        H: Into<String>,
        V: Into<String>,
    {
        let header: String = header.into();

        match self.get_mut(&header) {
            Some(values) => values.push(value.into()),
            None => self.insert(header, value),
        };
    }

    /// Appends many values to the given header.
    /// If the header does not exist, it will be created.
    /// If the header already exists, the values will be
    /// appended.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
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
    pub fn append_many<H, V>(&mut self, header: H, values: Vec<V>)
    where
        H: Into<String>,
        V: Into<String>,
    {
        let header: String = header.into();

        if let Some(current) = self.get_mut(&header) {
            let values = values.into_iter().map(|value| value.into());
            return current.extend(values);
        }

        self.insert_many(header, values);
    }

    /// Removes a given header.
    /// If the header does not exist, nothing will happen.
    /// If the header exists, all its values will be
    /// removed.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    /// use valar::http::Response;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::<Request>::from([("Content-Type", "application/json")]);
    ///
    /// headers.remove("Content-Type");
    ///
    /// assert_eq!(headers.get("Content-Type"), None);
    /// ```
    pub fn remove<H>(&mut self, header: H)
    where
        H: Into<String>,
    {
        let header: String = header.into();
        self.headers.remove(&header.to_lowercase());
    }

    /// Clears all the headers.
    /// After this operation, the headers will be empty.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Headers;
    /// use valar::http::Request;
    ///
    /// let mut headers: Headers<Request> =
    ///     Headers::from([("Content-Type", "application/json")]);
    ///
    /// headers.clear();
    ///
    /// assert_eq!(headers.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.headers.clear();
    }
}

impl Headers<Request> {
    /// Computes and creates the cookies based on the
    /// `Cookie` header.
    pub fn cookies(&self) -> Vec<Cookie<Request>> {
        match self.get("Cookie") {
            Some(values) => values
                .iter()
                .flat_map(|value| Cookie::<Request>::from_str(value))
                .collect(),
            None => vec![],
        }
    }

    pub fn cookie(&self, name: &str) -> Option<Cookie<Request>> {
        self.cookies()
            .into_iter()
            .find(|cookie| cookie.name() == name)
    }

    pub fn has_cookie(&self, name: &str) -> bool {
        self.cookie(name).is_some()
    }

    /// Sets the cookie using the `Cookie` header.
    pub fn set_cookie<C>(&mut self, cookie: C)
    where
        C: Into<Cookie<Request>>,
    {
        let cookie: Cookie<Request> = cookie.into();

        self.append("Cookie", cookie.to_string());
    }
}

impl Headers<Response> {
    /// Computes and creates the cookies based on the
    /// `Set-Cookie` header.
    pub fn cookies(&self) -> Vec<Cookie<Request>> {
        match self.get("Set-Cookie") {
            Some(values) => values
                .iter()
                .flat_map(|value| Cookie::<Request>::from_str(value))
                .collect(),
            None => vec![],
        }
    }

    pub fn cookie(&self, name: &str) -> Option<Cookie<Request>> {
        self.cookies()
            .into_iter()
            .find(|cookie| cookie.name() == name)
    }

    pub fn has_cookie(&self, name: &str) -> bool {
        self.cookie(name).is_some()
    }

    /// Sets the cookie using the `Set-Cookie` header.
    pub fn set_cookie<C>(&mut self, cookie: C)
    where
        C: Into<Cookie<Response>>,
    {
        let cookie: Cookie<Response> = cookie.into();

        self.append("Set-Cookie", cookie.to_string());
    }
}

impl<T> IntoIterator for Headers<T> {
    type IntoIter = IntoIter<Self::Item>;
    type Item = (String, String);

    /// Transformation into an iterator over the headers.
    fn into_iter(self) -> Self::IntoIter {
        let mut result: Vec<(String, String)> = Vec::new();

        for (header, values) in self.headers {
            for value in values {
                result.push((header.clone(), value));
            }
        }

        result.into_iter()
    }
}

impl<T> Extend<(String, String)> for Headers<T> {
    /// Extends the headers with the given iterator.
    fn extend<I: IntoIterator<Item = (String, String)>>(&mut self, iter: I) {
        for (header, value) in iter {
            self.append(header, value);
        }
    }
}

impl<T, H, V, const N: usize> From<[(H, V); N]> for Headers<T>
where
    H: Into<String>,
    V: Into<String>,
{
    fn from(value: [(H, V); N]) -> Self {
        Headers::<T>::from_iter(value)
    }
}

impl<T, H, V> FromIterator<(H, V)> for Headers<T>
where
    H: Into<String>,
    V: Into<String>,
{
    /// Creates a new `Headers` from an iterator.
    fn from_iter<I: IntoIterator<Item = (H, V)>>(iter: I) -> Self {
        let mut headers = Self::default();

        for (header, value) in iter {
            headers.append(header, value);
        }

        headers
    }
}

impl<T> TryFrom<Headers<T>> for HeaderMap {
    type Error = ConversionError;

    /// Converts the `Headers` into a `HeaderMap`.
    fn try_from(from: Headers<T>) -> Result<Self, Self::Error> {
        let mut headers = Self::new();

        for (header, value) in from {
            let header = HeaderName::from_str(&header)?;
            let value = HeaderValue::from_str(&value)?;

            headers.append(header.clone(), value);
        }

        Ok(headers)
    }
}
