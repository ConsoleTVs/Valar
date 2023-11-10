use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::marker::PhantomData;
use std::str::FromStr;

use thiserror::Error as ThisError;

use crate::http::Request;
use crate::http::Response;

/// An error that occurs when parsing a cookie.
/// This error is returned when the cookie string
/// is not formatted correctly.
#[derive(ThisError, Debug)]
#[error("There was an error parsing the cookie")]
pub struct Error;

#[derive(Debug, PartialEq, Eq)]
pub enum SameSite {
    /// The cookie will only be sent in a first-party
    /// context and not be sent along with requests
    /// initiated by third party websites.
    Strict,

    /// The cookie will be sent in all contexts, i.e.
    /// both first-party and third-party contexts.
    Lax,

    /// The cookie will be sent in all contexts, i.e.
    /// both first-party and third-party contexts.
    /// However, the cookie will not be sent along with
    /// top-level navigations.
    ///
    /// This value is intended to provide some protection
    /// against cross-site request forgery attacks, while
    /// maintaining compatibility with unpatched user
    /// agents.
    None,
}

impl Display for SameSite {
    /// Formats the `SameSite` value as a string.
    /// This is used when serializing the cookie.
    /// The value will be one of the following:
    /// - `Strict`
    /// - `Lax`
    /// - `None`
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

#[derive(Debug)]
pub struct Cookie<T> {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<u64>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
    _marker: PhantomData<T>,
}

impl<T> Cookie<T> {
    /// Creates a new cookie builder.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = Cookie::builder("name", "value").build();
    ///
    /// assert_eq!(cookie.name(), "name");
    /// assert_eq!(cookie.value(), "value");
    /// ```
    pub fn builder<N, V>(name: N, value: V) -> CookieBuilder<T>
    where
        N: Into<String>,
        V: Into<String>,
    {
        CookieBuilder::new(name, value)
    }

    /// Returns the cookie name.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = Cookie::builder("name", "value").build();
    ///
    /// assert_eq!(cookie.name(), "name");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the cookie value.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = Cookie::builder("name", "value").build();
    ///
    /// assert_eq!(cookie.value(), "value");
    /// ```
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Cookie<Response> {
    /// Returns the cookie path.
    /// If the path is not insert, this will return `None`.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> =
    ///     Cookie::builder("name", "value").path(Some("/path")).build();
    ///
    /// assert_eq!(cookie.path(), Some("/path"));
    /// ```
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the cookie domain.
    /// If the domain is not insert, this will return
    /// `None`.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = Cookie::builder("name", "value")
    ///     .domain(Some("example.com"))
    ///     .build();
    ///
    /// assert_eq!(cookie.domain(), Some("example.com"));
    /// ```
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Returns the cookie max age.
    /// If the max age is not insert, this will return
    /// `None`. The max age is the number of seconds
    /// until the cookie expires.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> =
    ///     Cookie::builder("name", "value").max_age(Some(3600)).build();
    ///
    /// assert_eq!(cookie.max_age(), Some(&3600));
    /// ```
    pub fn max_age(&self) -> Option<&u64> {
        self.max_age.as_ref()
    }

    /// Returns whether the cookie is secure.
    /// If the cookie is secure, it will only be sent over
    /// HTTPS connections.
    /// If the cookie is not secure, it will be sent over
    /// both HTTP and HTTPS connections.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> =
    ///     Cookie::builder("name", "value").secure(true).build();
    ///
    /// assert_eq!(cookie.secure(), true);
    /// ```
    pub fn secure(&self) -> bool {
        self.secure
    }

    /// Returns whether the cookie is HTTP only.
    /// If the cookie is HTTP only, it will not be
    /// accessible to JavaScript.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> =
    ///     Cookie::builder("name", "value").http_only(true).build();
    ///
    /// assert_eq!(cookie.http_only(), true);
    /// ```
    pub fn http_only(&self) -> bool {
        self.http_only
    }

    /// Returns the cookie same site.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::SameSite;
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = Cookie::builder("name", "value")
    ///     .same_site(Some(SameSite::Strict))
    ///     .build();
    ///
    /// assert_eq!(cookie.same_site(), Some(&SameSite::Strict));
    /// ```
    pub fn same_site(&self) -> Option<&SameSite> {
        self.same_site.as_ref()
    }
}

pub struct CookieBuilder<T> {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<u64>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
    _marker: PhantomData<T>,
}

impl<T> CookieBuilder<T> {
    /// Creates a new cookie builder.
    /// The cookie name and value are required.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = CookieBuilder::new("name", "value").build();
    ///
    /// assert_eq!(cookie.name(), "name");
    /// assert_eq!(cookie.value(), "value");
    /// ```
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        Self {
            name: name.into(),
            value: value.into(),
            path: None,
            domain: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
            _marker: PhantomData::<T>,
        }
    }

    /// Sets the cookie name and returns the builder.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> =
    ///     CookieBuilder::new("name", "value").name("new_name").build();
    ///
    /// assert_eq!(cookie.name(), "new_name");
    /// ```
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<String>,
    {
        self.name = name.into();

        self
    }

    /// Sets the cookie value and returns the builder.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = CookieBuilder::new("name", "value")
    ///     .value("new_value")
    ///     .build();
    ///
    /// assert_eq!(cookie.value(), "new_value");
    /// ```
    pub fn value<V>(mut self, value: V) -> Self
    where
        V: Into<String>,
    {
        self.value = value.into();

        self
    }

    /// Builds the cookie.
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    /// use valar::http::Cookie;
    /// use valar::http::Response;
    ///
    /// let cookie: Cookie<Response> = CookieBuilder::new("name", "value").build();
    ///
    /// assert_eq!(cookie.name(), "name");
    /// assert_eq!(cookie.value(), "value");
    /// ```
    pub fn build(self) -> Cookie<T>
    where
        Self: Into<Cookie<T>>,
    {
        self.into()
    }
}

impl CookieBuilder<Response> {
    /// Sets the cookie name and returns the builder.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    ///
    /// let cookie = CookieBuilder::new("name", "value")
    ///     .path(Some("/path"))
    ///     .build();
    ///
    /// assert_eq!(cookie.path(), Some("/path"));
    /// ```
    pub fn path<T>(mut self, path: Option<T>) -> Self
    where
        T: Into<String>,
    {
        self.path = path.map(|p| p.into());

        self
    }

    /// Sets the cookie domain and returns the builder.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    ///
    /// let cookie = CookieBuilder::new("name", "value")
    ///     .domain(Some("example.com"))
    ///     .build();
    ///
    /// assert_eq!(cookie.domain(), Some("example.com"));
    /// ```
    pub fn domain<T>(mut self, domain: Option<T>) -> Self
    where
        T: Into<String>,
    {
        self.domain = domain.map(|d| d.into());

        self
    }

    /// Sets the cookie max age and returns the builder.
    /// The max age is the number of seconds until the
    /// cookie expires.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    ///
    /// let cookie = CookieBuilder::new("name", "value")
    ///     .max_age(Some(3600))
    ///     .build();
    ///
    /// assert_eq!(cookie.max_age(), Some(&3600));
    /// ```
    pub fn max_age(mut self, max_age: Option<u64>) -> Self {
        self.max_age = max_age;

        self
    }

    /// Sets whether the cookie is secure and returns the
    /// builder. If the cookie is secure, it will only
    /// be sent over HTTPS connections.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    ///
    /// let cookie = CookieBuilder::new("name", "value").secure(true).build();
    ///
    /// assert_eq!(cookie.secure(), true);
    /// ```
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;

        self
    }

    /// Sets whether the cookie is HTTP only and returns the
    /// builder. If the cookie is HTTP only, it will not be
    /// accessible to JavaScript.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    ///
    /// let cookie = CookieBuilder::new("name", "value").http_only(true).build();
    ///
    /// assert_eq!(cookie.http_only(), true);
    /// ```
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;

        self
    }

    /// Sets the cookie same site and returns the builder.
    /// The same site attribute is used to prevent CSRF
    /// attacks. The same site attribute can be `None`,
    /// `Lax`, or `Strict`. If the same site attribute
    /// is `None`, the cookie will not have the same
    /// site attribute.
    ///
    /// # Example
    /// ```no_run
    /// use valar::http::cookie::CookieBuilder;
    /// use valar::http::cookie::SameSite;
    ///
    /// let cookie = CookieBuilder::new("name", "value")
    ///     .same_site(Some(SameSite::Lax))
    ///     .build();
    ///
    /// assert_eq!(cookie.same_site(), Some(&SameSite::Lax));
    /// ```
    pub fn same_site<T>(mut self, same_site: Option<T>) -> Self
    where
        T: Into<SameSite>,
    {
        self.same_site = same_site.map(|site| site.into());

        self
    }
}

impl<App: Send + Sync + 'static> From<CookieBuilder<Request<App>>> for Cookie<Request<App>> {
    /// Converts the cookie builder into a cookie.
    fn from(builder: CookieBuilder<Request<App>>) -> Self {
        Self {
            name: builder.name,
            value: builder.value,
            path: builder.path,
            domain: builder.domain,
            max_age: builder.max_age,
            secure: builder.secure,
            http_only: builder.http_only,
            same_site: builder.same_site,
            _marker: PhantomData::<Request<App>>,
        }
    }
}

impl From<CookieBuilder<Response>> for Cookie<Response> {
    /// Converts the cookie builder into a cookie.
    fn from(builder: CookieBuilder<Response>) -> Self {
        Self {
            name: builder.name,
            value: builder.value,
            path: builder.path,
            domain: builder.domain,
            max_age: builder.max_age,
            secure: builder.secure,
            http_only: builder.http_only,
            same_site: builder.same_site,
            _marker: PhantomData::<Response>,
        }
    }
}

impl FromStr for Cookie<Response> {
    type Err = Error;

    /// It will only process a single cookie. Multiple
    /// cookies sent must first be splitted acordingly.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut iter = string.trim().split('=');

        let name: &str = iter.next().ok_or(Self::Err {})?;
        let name = name.trim();
        let value: &str = iter.next().ok_or(Self::Err {})?;
        let value = value.trim();

        let mut iter = value.trim().split(';');
        let value: &str = iter.next().ok_or(Self::Err {})?;

        let cookie = Cookie::builder(name, value);

        Ok(cookie.build())
    }
}

impl<App: Send + Sync + 'static> FromStr for Cookie<Request<App>> {
    type Err = Error;

    /// It will only process a single cookie. Multiple
    /// cookies sent must first be splitted acordingly.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut iter = string.trim().split('=');

        let name: &str = iter.next().ok_or(Self::Err {})?;
        let name = name.trim();
        let value: &str = iter.next().ok_or(Self::Err {})?;
        let value = value.trim();

        let cookie = Cookie::builder(name, value);

        Ok(cookie.build())
    }
}

impl<App: Send + Sync + 'static> Display for Cookie<Request<App>> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}={}", self.name(), self.value())
    }
}

impl Display for Cookie<Response> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}={}", self.name(), self.value())?;

        if let Some(path) = &self.path {
            write!(f, "; Path={}", path)?;
        }

        if let Some(domain) = &self.domain {
            write!(f, "; Domain={}", domain)?;
        }

        if let Some(max_age) = &self.max_age {
            write!(f, "; Max-Age={}", max_age)?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(same_site) = &self.same_site {
            write!(f, "; SameSite={}", same_site)?;
        }

        Ok(())
    }
}

// impl Cookie for RequestCookie {
//     fn name(&self) -> &str {
//         &self.name
//     }

//     fn value(&self) -> &str {
//         &self.value
//     }

//     fn many_from_str(string: &str) -> Result<Vec<Self>,
// CookieParseError> {         let mut cookies = Vec::new();

//         for cookie in string.split(';') {
//             cookies.push(cookie.trim().parse()?);
//         }

//         Ok(cookies)
//     }
// }

// pub trait Cookie: FromStr<Err = CookieParseError> {
//     fn name(&self) -> &str;
//     fn value(&self) -> &str;

//     fn many_from_str(string: &str) -> Result<Vec<Self>,
// CookieParseError> {         string
//             .split(';')
//             .map(|cookie| cookie.trim().parse())
//             .collect()
//     }
// }

// pub trait HasCookies: HasHeaders {
//     type Item: Cookie;

//     fn cookies(&self) -> Result<Vec<Self::Item>,
// CookieParseError> {         match
// self.headers().get("Cookie") {             Some(values)
// => {                 let mut cookies = Vec::new();

//                 for value in values {
//                     for cookie in
// Self::Item::many_from_str(value)? {
// cookies.push(cookie);                     }
//                 }

//                 Ok(cookies)
//             }
//             None => Ok(vec![]),
//         }
//     }

//     fn cookie(&self, name: &str) -> Option<Self::Item> {
//         self.cookies()
//             .ok()?
//             .into_iter()
//             .find(|cookie| cookie.name() == name)
//     }

//     fn has_cookie(&self, name: &str) -> bool {
//         self.cookie(name).is_some()
//     }
// }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::http::cookie::Cookie;
    use crate::http::cookie::SameSite;
    use crate::http::Request;

    #[test]
    fn it_can_create_simple_cookies() {
        struct App;
        let cookie = Cookie::<Request<App>>::builder("foo", "bar").build();

        assert_eq!(cookie.to_string(), "foo=bar");
    }

    #[test]
    fn it_can_create_complex_cookies() {
        let cookie = Cookie::builder("foo", "bar")
            .path(Some("/"))
            .domain(Some("example.com"))
            .max_age(Some(3600))
            .secure(true)
            .http_only(true)
            .same_site(Some(SameSite::Strict))
            .build();

        assert_eq!(
            cookie.to_string(),
            "foo=bar; Path=/; Domain=example.com; Max-Age=3600; Secure; HttpOnly; SameSite=Strict"
        );
    }

    #[test]
    fn it_can_parse_simple_cookies() {
        struct App;
        let cookie = Cookie::<Request<App>>::from_str("foo=bar").unwrap();

        assert_eq!(cookie.name(), "foo");
        assert_eq!(cookie.value(), "bar");
    }

    #[test]
    #[ignore]
    fn it_can_parse_complex_cookies() {
        let cookie = Cookie::from_str(
            "foo=bar; Path=/; Domain=example.com; Max-Age=3600; Secure; HttpOnly; SameSite=Strict",
        )
        .unwrap();

        assert_eq!(cookie.name(), "foo");
        assert_eq!(cookie.value(), "bar");
        assert_eq!(cookie.domain(), Some("example.com"));
        assert_eq!(cookie.max_age(), Some(&3600));
        assert!(cookie.secure());
        assert!(cookie.http_only());
        assert_eq!(cookie.same_site(), Some(&SameSite::Strict));
    }
}
