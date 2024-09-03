use std::fmt::{self, Debug, Formatter};

#[derive(Default)]
pub struct Password(Option<String>);

impl Debug for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(password) = &self.0 {
            write!(f, "Password({})", "*".repeat(password.len()))
        } else {
            write!(f, "Password(None)")
        }
    }
}

impl From<Option<String>> for Password {
    fn from(password: Option<String>) -> Self {
        Self(password)
    }
}

impl From<&str> for Password {
    fn from(password: &str) -> Self {
        Self(Some(password.to_string()))
    }
}

impl AsRef<Option<String>> for Password {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_and_as_ref() {
        let password: Password = None.into();
        assert_eq!(password.as_ref(), &None);

        let password: Password = "password".into();
        assert_eq!(password.as_ref(), &Some("password".to_string()));
    }

    #[test]
    fn test_debug() {
        let password: Password = None.into();
        assert_eq!(format!("{:?}", password), "Password(None)");

        let password: Password = "password".into();
        assert_eq!(format!("{:?}", password), "Password(********)");
    }
}
