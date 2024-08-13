#![allow(unused)]

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use derive_setters::Setters;
use genai::adapter::AdapterKind;

#[derive(Clone)]
pub struct Model(&'static str);

pub mod open_ai {
    use super::*;
    pub const GPT3_5_TURBO: Model = Model("gp-3.5-turbo");
    pub const GPT4: Model = Model("gpt-4");
    pub const GPT4_TURBO: Model = Model("gpt-4-turbo");
    pub const GPT4O_MINI: Model = Model("gpt-4o-mini");
    pub const GPT4O: Model = Model("gpt-4o");
}

pub mod ollama {
    use super::*;
    pub const GEMMA2B: Model = Model("gemma:2b");
}

pub mod anthropic {
    use super::*;
    pub const CLAUDE3_HAIKU_20240307: Model = Model("claude-3-haiku-20240307");
    pub const CLAUDE3_SONNET_20240229: Model = Model("claude-3-sonnet-20240229");
    pub const CLAUDE3_OPUS_20240229: Model = Model("claude-3-opus-20240229");
    pub const CLAUDE35_SONNET_20240620: Model = Model("claude-3-5-sonnet-20240620");
}

pub mod cohere {
    use super::*;
    pub const COMMAND_LIGHT_NIGHTLY: Model = Model("command-light-nightly");
    pub const COMMAND_LIGHT: Model = Model("command-light");
    pub const COMMAND_NIGHTLY: Model = Model("command-nightly");
    pub const COMMAND: Model = Model("command");
    pub const COMMAND_R: Model = Model("command-r");
    pub const COMMAND_R_PLUS: Model = Model("command-r-plus");
}

pub mod gemini {
    use super::*;
    pub const GEMINI15_FLASH_LATEST: Model = Model("gemini-1.5-flash-latest");
    pub const GEMINI10_PRO: Model = Model("gemini-1.0-pro");
    pub const GEMINI15_FLASH: Model = Model("gemini-1.5-flash");
    pub const GEMINI15_PRO: Model = Model("gemini-1.5-pro");
}

pub mod groq {
    use super::*;
    pub const LLAMA708192: Model = Model("llama3-70b-8192");
    pub const LLAMA38192: Model = Model("llama3-8b-8192");
    pub const LLAMA_GROQ8B8192_TOOL_USE_PREVIEW: Model =
        Model("llama3-groq-8b-8192-tool-use-preview");
    pub const LLAMA_GROQ70B8192_TOOL_USE_PREVIEW: Model =
        Model("llama3-groq-70b-8192-tool-use-preview");
    pub const GEMMA29B_IT: Model = Model("gemma2-9b-it");
    pub const GEMMA7B_IT: Model = Model("gemma-7b-it");
    pub const MIXTRAL_8X7B32768: Model = Model("mixtral-8x7b-32768");
    pub const LLAMA8B_INSTANT: Model = Model("llama-3.1-8b-instant");
    pub const LLAMA70B_VERSATILE: Model = Model("llama-3.1-70b-versatile");
    pub const LLAMA405B_REASONING: Model = Model("llama-3.1-405b-reasoning");
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}
