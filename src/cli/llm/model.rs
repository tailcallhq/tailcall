#![allow(unused)]
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use genai::adapter::AdapterKind;

#[derive(Clone)]
pub struct Model(&'static str);

pub struct OpenAI;
pub struct Ollama;
pub struct Anthropic;
pub struct Cohere;
pub struct Gemini;
pub struct Groq;

impl Model {
    pub const OPEN_AI: OpenAI = OpenAI;
    pub const OLLAMA: Ollama = Ollama;
    pub const ANTHROPIC: Anthropic = Anthropic;
    pub const COHERE: Cohere = Cohere;
    pub const GEMINI: Gemini = Gemini;
    pub const GROQ: Groq = Groq;

    pub fn inner(&self) -> &'static str {
        self.0
    }
    pub fn to_adapter_kind(&self) -> genai::adapter::AdapterKind {
        // should be safe to call unwrap here
        AdapterKind::from_model(self.0).unwrap()
    }
}

impl OpenAI {
    const GPT4O: Model = Model("gpt-4o");
    const GPT4O_MINI: Model = Model("gpt-4o-mini");
    const GPT4_TURBO: Model = Model("gpt-4-turbo");
    const GPT4: Model = Model("gpt-4");
    const GPT35_TURBO: Model = Model("gpt-3.5-turbo");
    pub fn gpt3_5_turbo(&self) -> Model {
        Self::GPT35_TURBO
    }
    pub fn gpt4(&self) -> Model {
        Self::GPT4
    }
    pub fn gpt4_turbo(&self) -> Model {
        Self::GPT4_TURBO
    }
    pub fn gpt4o_mini(&self) -> Model {
        Self::GPT4O_MINI
    }
    pub fn gpt4o(&self) -> Model {
        Self::GPT4O
    }
}
impl Ollama {
    const GEMMA2B: Model = Model("gemma:2b");
    pub fn gemma2b(&self) -> Model {
        Self::GEMMA2B
    }
}
impl Anthropic {
    const CLAUDE35_SONNET_20240620: Model = Model("claude-3-5-sonnet-20240620");
    const CLAUDE3_OPUS_20240229: Model = Model("claude-3-opus-20240229");
    const CLAUDE3_SONNET_20240229: Model = Model("claude-3-sonnet-20240229");
    const CLAUDE3_HAIKU_20240307: Model = Model("claude-3-haiku-20240307");
    pub fn claude3_haiku_20240307(&self) -> Model {
        Self::CLAUDE3_HAIKU_20240307
    }
    pub fn claude3_sonnet_20240229(&self) -> Model {
        Self::CLAUDE3_SONNET_20240229
    }
    pub fn claude3_opus_20240229(&self) -> Model {
        Self::CLAUDE3_OPUS_20240229
    }
    pub fn claude35_sonnet_20240620(&self) -> Model {
        Self::CLAUDE35_SONNET_20240620
    }
}

impl Cohere {
    const COMMAND_R_PLUS: Model = Model("command-r-plus");
    const COMMAND_R: Model = Model("command-r");
    const COMMAND: Model = Model("command");
    const COMMAND_NIGHTLY: Model = Model("command-nightly");
    const COMMAND_LIGHT: Model = Model("command-light");
    const COMMAND_LIGHT_NIGHTLY: Model = Model("command-light-nightly");
    pub fn command_light_nightly(&self) -> Model {
        Self::COMMAND_LIGHT_NIGHTLY
    }
    pub fn command_light(&self) -> Model {
        Self::COMMAND_LIGHT
    }
    pub fn command_nightly(&self) -> Model {
        Self::COMMAND_NIGHTLY
    }
    pub fn command(&self) -> Model {
        Self::COMMAND
    }
    pub fn command_r(&self) -> Model {
        Self::COMMAND_R
    }
    pub fn command_r_plus(&self) -> Model {
        Self::COMMAND_R_PLUS
    }
}

impl Gemini {
    const GEMINI15_PRO: Model = Model("gemini-1.5-pro");
    const GEMINI15_FLASH: Model = Model("gemini-1.5-flash");
    const GEMINI10_PRO: Model = Model("gemini-1.0-pro");
    const GEMINI15_FLASH_LATEST: Model = Model("gemini-1.5-flash-latest");
    pub fn gemini15_flash_latest(&self) -> Model {
        Self::GEMINI15_FLASH_LATEST
    }
    pub fn gemini10_pro(&self) -> Model {
        Self::GEMINI10_PRO
    }
    pub fn gemini15_flash(&self) -> Model {
        Self::GEMINI15_FLASH
    }
    pub fn gemini15_pro(&self) -> Model {
        Self::GEMINI15_PRO
    }
}

impl Groq {
    const LLAMA405B_REASONING: Model = Model("llama-3.1-405b-reasoning");
    const LLAMA70B_VERSATILE: Model = Model("llama-3.1-70b-versatile");
    const LLAMA8B_INSTANT: Model = Model("llama-3.1-8b-instant");
    const MIXTRAL_8X7B32768: Model = Model("mixtral-8x7b-32768");
    const GEMMA7B_IT: Model = Model("gemma-7b-it");
    const GEMMA29B_IT: Model = Model("gemma2-9b-it");
    const LLAMA_GROQ70B8192_TOOL_USE_PREVIEW: Model =
        Model("llama3-groq-70b-8192-tool-use-preview");
    const LLAMA_GROQ8B8192_TOOL_USE_PREVIEW: Model = Model("llama3-groq-8b-8192-tool-use-preview");
    const LLAMA38192: Model = Model("llama3-8b-8192");
    const LLAMA708192: Model = Model("llama3-70b-8192");
    pub fn llama708192(&self) -> Model {
        Self::LLAMA708192
    }
    pub fn llama38192(&self) -> Model {
        Self::LLAMA38192
    }
    pub fn llama_groq8b8192_tool_use_preview(&self) -> Model {
        Self::LLAMA_GROQ8B8192_TOOL_USE_PREVIEW
    }
    pub fn llama_groq70b8192_tool_use_preview(&self) -> Model {
        Self::LLAMA_GROQ70B8192_TOOL_USE_PREVIEW
    }
    pub fn gemma29b_it(&self) -> Model {
        Self::GEMMA29B_IT
    }
    pub fn gemma7b_it(&self) -> Model {
        Self::GEMMA7B_IT
    }
    pub fn mixtral_8x7b32768(&self) -> Model {
        Self::MIXTRAL_8X7B32768
    }
    pub fn llama8b_instant(&self) -> Model {
        Self::LLAMA8B_INSTANT
    }
    pub fn llama70b_versatile(&self) -> Model {
        Self::LLAMA70B_VERSATILE
    }
    pub fn llama405b_reasoning(&self) -> Model {
        Self::LLAMA405B_REASONING
    }
}
