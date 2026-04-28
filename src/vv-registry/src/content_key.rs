use std::{error::Error, fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentKey {
    namespace: String,
    name: String,
}

impl ContentKey {
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, ContentKeyParseError> {
        let key = Self {
            namespace: namespace.into(),
            name: name.into(),
        };
        key.validate()?;
        Ok(key)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<(), ContentKeyParseError> {
        validate_part(&self.namespace, "namespace")?;
        validate_part(&self.name, "name")?;
        Ok(())
    }
}

impl fmt::Display for ContentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

impl FromStr for ContentKey {
    type Err = ContentKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (namespace, name) = value
            .split_once(':')
            .ok_or_else(|| ContentKeyParseError::new(value, "expected namespace:name"))?;
        ContentKey::new(namespace, name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentKeyParseError {
    value: String,
    reason: &'static str,
}

impl ContentKeyParseError {
    fn new(value: impl Into<String>, reason: &'static str) -> Self {
        Self {
            value: value.into(),
            reason,
        }
    }
}

impl fmt::Display for ContentKeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid content key `{}`: {}", self.value, self.reason)
    }
}

impl Error for ContentKeyParseError {}

fn validate_part(value: &str, label: &'static str) -> Result<(), ContentKeyParseError> {
    if value.is_empty() {
        return Err(ContentKeyParseError::new(value, label));
    }

    let valid = value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'/')
    });
    if valid {
        Ok(())
    } else {
        Err(ContentKeyParseError::new(
            value,
            "allowed characters are lowercase ascii, digits, _, -, and /",
        ))
    }
}
