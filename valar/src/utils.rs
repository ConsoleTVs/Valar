pub trait TruncatableToFit {
    fn truncate_to_fit(self, width: usize) -> String;
}

impl TruncatableToFit for String {
    fn truncate_to_fit(mut self, width: usize) -> String {
        if self.len() <= width {
            return self;
        }

        self.truncate(width - 3);
        self.push_str("...");

        self
    }
}

impl TruncatableToFit for &str {
    fn truncate_to_fit(self, width: usize) -> String {
        let mut result = self.to_string();

        if self.len() <= width {
            return result;
        }

        result.truncate(width - 3);
        result.push_str("...");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_truncate_to_fit_str() {
        assert_eq!("/foo/bar/baz".truncate_to_fit(10), "/foo/ba...");
        assert_eq!("/foo/bar/baz".truncate_to_fit(100), "/foo/bar/baz");
    }

    #[test]
    fn it_can_truncate_to_fit_strings() {
        assert_eq!("/foo/bar/baz".to_string().truncate_to_fit(10), "/foo/ba...");
        assert_eq!(
            "/foo/bar/baz".to_string().truncate_to_fit(100),
            "/foo/bar/baz"
        );
    }
}
