use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl Error {
    pub fn msg(m: impl Into<String>) -> Self {
        Self(
            m.into()
                .chars()
                .map(|c| if c.is_control() { '?' } else { c })
                .collect(),
        )
    }
    pub fn message(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn control_chars_never_reach_the_message() {
        let e = Error::msg("x\u{1b}[31m\u{7}y\nz\u{85}");
        assert_eq!(e.message(), "x?[31m?y?z?");
    }
}
