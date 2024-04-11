use std::fmt::Display;

impl Display for crate::error::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pretty_print(f, true)
    }
}
