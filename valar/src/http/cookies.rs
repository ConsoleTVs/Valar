use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::FromStr;

pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Display for SameSite {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

#[derive(Default)]
pub struct Cookie {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    // expires: Option<DateTime<Utc>>,
    max_age: Option<u64>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

pub type Cookies = Vec<Cookie>;

impl Cookie {
    pub fn new<T, K>(name: T, value: K) -> Self
    where
        T: Into<String>,
        K: Into<String>,
    {
        Self {
            name: name.into(),
            value: value.into(),
            ..Self::default()
        }
    }

    pub fn path<T>(mut self, path: Option<T>) -> Self
    where
        T: Into<String>,
    {
        self.path = path.map(|p| p.into());

        self
    }

    pub fn domain<T>(mut self, domain: Option<T>) -> Self
    where
        T: Into<String>,
    {
        self.domain = domain.map(|d| d.into());

        self
    }

    pub fn max_age(mut self, max_age: Option<u64>) -> Self {
        self.max_age = max_age;

        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;

        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;

        self
    }

    pub fn same_site(mut self, same_site: Option<SameSite>) -> Self {
        self.same_site = same_site;

        self
    }

    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<String>,
    {
        self.name = name.into();

        self
    }

    pub fn value<T>(mut self, value: T) -> Self
    where
        T: Into<String>,
    {
        self.value = value.into();

        self
    }
}

impl FromStr for Cookie {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!();
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}={}", self.name, self.value)?;

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

#[cfg(test)]
mod tests {
    use crate::http::cookies::Cookie;
    use crate::http::cookies::SameSite;

    #[test]
    fn test_cookie() {
        let cookie = Cookie::new("foo", "bar")
            .path(Some("/"))
            .domain(Some("example.com"))
            .max_age(Some(3600))
            .secure(true)
            .http_only(true)
            .same_site(Some(SameSite::Strict));

        assert_eq!(
            cookie.to_string(),
            "foo=bar; Path=/; Domain=example.com; Max-Age=3600; Secure; HttpOnly; \
             SameSite=Strict"
        );
    }
}
