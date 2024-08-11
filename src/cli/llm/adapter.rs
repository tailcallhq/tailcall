#![allow(unused)]
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Adapter {
    Groq(GroqModel),
    OpenAI(OpenAIModel),
    Ollama(OllamaModel),
    Anthropic(AnthropicModel),
    Cohere(CohereModel),
    Gemini(GeminiModel),
}

#[derive(Clone)]
pub enum GroqModel {
    Llama3405bReasoning,
    Llama370bVersatile,
    Llama38bInstant,
    Mixtral8x7b32768,
    Gemma7bIt,
    Gemma29bIt,
    Llama3Groq70b8192ToolUsePreview,
    Llama3Groq8b8192ToolUsePreview,
    Llama38b192,
    Llama370b8192,
}

#[derive(Clone)]
pub enum GeminiModel {
    Gemini15Pro,
    Gemini15Flash,
    Gemini10Pro,
    Gemini15FlashLatest,
}

#[derive(Clone)]
pub enum OpenAIModel {
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    Gpt4,
    Gpt35Turbo,
}

#[derive(Clone)]
pub enum AnthropicModel {
    Claude35Sonnet20240620,
    Claude3Opus20240229,
    Claude3Sonnet20240229,
    Claude3Haiku20240307,
}

#[derive(Clone)]
pub enum CohereModel {
    CommandRPlus,
    CommandR,
    Command,
    CommandNightly,
    CommandLight,
    CommandLightNightly,
}

#[derive(Clone)]
pub enum OllamaModel {
    Gemma2b,
}

impl Display for Adapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for GroqModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            GroqModel::Llama3405bReasoning => "llama-3.1-405b-reasoning",
            GroqModel::Llama370bVersatile => "llama-3.1-70b-versatile",
            GroqModel::Llama38bInstant => "llama-3.1-8b-instant",
            GroqModel::Mixtral8x7b32768 => "mixtral-8x7b-32768",
            GroqModel::Gemma7bIt => "gemma-7b-it",
            GroqModel::Gemma29bIt => "gemma2-9b-it",
            GroqModel::Llama3Groq70b8192ToolUsePreview => "llama3-groq-70b-8192-tool-use-preview",
            GroqModel::Llama3Groq8b8192ToolUsePreview => "llama3-groq-8b-8192-tool-use-preview",
            GroqModel::Llama38b192 => "llama3-8b-8192",
            GroqModel::Llama370b8192 => "llama3-70b-8192",
        };
        write!(f, "{}", model_name)
    }
}

impl Display for GeminiModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            GeminiModel::Gemini15Pro => "gemini-1.5-pro",
            GeminiModel::Gemini15Flash => "gemini-1.5-flash",
            GeminiModel::Gemini10Pro => "gemini-1.0-pro",
            GeminiModel::Gemini15FlashLatest => "gemini-1.5-flash-latest",
        };
        write!(f, "{}", model_name)
    }
}

impl Display for OpenAIModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            OpenAIModel::Gpt4o => "gpt-4o",
            OpenAIModel::Gpt4oMini => "gpt-4o-mini",
            OpenAIModel::Gpt4Turbo => "gpt-4-turbo",
            OpenAIModel::Gpt4 => "gpt-4",
            OpenAIModel::Gpt35Turbo => "gpt-3.5-turbo",
        };
        write!(f, "{}", model_name)
    }
}

impl Display for OllamaModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            OllamaModel::Gemma2b => "gemma:2b",
        };
        write!(f, "{}", model_name)
    }
}

impl Display for AnthropicModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            AnthropicModel::Claude35Sonnet20240620 => "claude-3-5-sonnet-20240620",
            AnthropicModel::Claude3Opus20240229 => "claude-3-opus-20240229",
            AnthropicModel::Claude3Sonnet20240229 => "claude-3-sonnet-20240229",
            AnthropicModel::Claude3Haiku20240307 => "claude-3-haiku-20240307",
        };
        write!(f, "{}", model_name)
    }
}

impl Display for CohereModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            CohereModel::CommandRPlus => "command-r-plus",
            CohereModel::CommandR => "command-r",
            CohereModel::Command => "command",
            CohereModel::CommandNightly => "command-nightly",
            CohereModel::CommandLight => "command-light",
            CohereModel::CommandLightNightly => "command-light-nightly",
        };
        write!(f, "{}", model_name)
    }
}
