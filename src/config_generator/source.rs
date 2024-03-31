use crate::config::UnsupportedFileFormat;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GeneratorSource {
    PROTO,
}

const ALL: &[GeneratorSource] = &[GeneratorSource::PROTO];

const PROTO_EXT: &str = "proto";

impl std::str::FromStr for GeneratorSource {
    type Err = UnsupportedFileFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(GeneratorSource::PROTO),
            _ => Err(UnsupportedFileFormat(s.to_string())),
        }
    }
}

impl GeneratorSource {
    pub fn ext(&self) -> &'static str {
        match self {
            GeneratorSource::PROTO => PROTO_EXT,
        }
    }
    fn ends_with(&self, content: &str) -> bool {
        content.ends_with(&format!(".{}", self.ext()))
    }
    pub fn detect(name: &str) -> Result<GeneratorSource, UnsupportedFileFormat> {
        ALL.iter()
            .find(|format| format.ends_with(name))
            .ok_or(UnsupportedFileFormat(name.to_string()))
            .cloned()
    }
}
