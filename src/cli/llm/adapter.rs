use std::fmt::{Display, Formatter};
#[derive(Clone)]
pub enum Adapter {
    Groq(GroqModel),
}

#[derive(Clone)]
pub enum GroqModel {
    Llama31_405bReasoning,
    Llama31_70bVersatile,
    Llama31_8bInstant,
    Mixtral8x7b32768,
    Gemma7bIt,
    Gemma2_9bIt,
    Llama3Groq70b8192ToolUsePreview,
    Llama3Groq8b8192ToolUsePreview,
    Llama38b8192,
    Llama370b8192,
}

impl Display for Adapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let provider_name = match self {
            Adapter::Groq(provider) => provider.to_string(),
        };
        write!(f, "{}", provider_name)
    }
}

impl Display for GroqModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            GroqModel::Llama31_405bReasoning => "llama-3.1-405b-reasoning",
            GroqModel::Llama31_70bVersatile => "llama-3.1-70b-versatile",
            GroqModel::Llama31_8bInstant => "llama-3.1-8b-instant",
            GroqModel::Mixtral8x7b32768 => "mixtral-8x7b-32768",
            GroqModel::Gemma7bIt => "gemma-7b-it",
            GroqModel::Gemma2_9bIt => "gemma2-9b-it",
            GroqModel::Llama3Groq70b8192ToolUsePreview => "llama3-groq-70b-8192-tool-use-preview",
            GroqModel::Llama3Groq8b8192ToolUsePreview => "llama3-groq-8b-8192-tool-use-preview",
            GroqModel::Llama38b8192 => "llama3-8b-8192",
            GroqModel::Llama370b8192 => "llama3-70b-8192",
        };
        write!(f, "{}", model_name)
    }
}
