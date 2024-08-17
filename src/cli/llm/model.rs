#![allow(unused)]

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use derive_setters::Setters;
use genai::adapter::AdapterKind;
use rand::rngs::adapter;

#[derive(Clone)]
pub enum Model {
    Claude35Sonnet,
    Claude3Haiku,
    Claude3Opus,
    Claude3Sonnet,
    Command,
    CommandLight,
    CommandLightNightly,
    CommandNightly,
    CommandR,
    CommandRPlus,
    Gemini10Pro,
    Gemini15Flash,
    Gemini15FlashLatest,
    Gemini15Pro,
    Gemma9b,
    Gemma2b,
    Gemma7b,
    Gpt35Turbo,
    Gpt4,
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    Llama405bReasoning,
    Llama70b,
    Llama70bVersatile,
    Llama8bInstant,
    LlamaGroq8b,
    Llama8b,
    LlamaGroq70bToolUsePreview,
    Mixtral7b,
}

impl Model {
    fn info(&self) -> Info {
        match &self {
            Model::Claude35Sonnet => Info {
                //
                name: "claude-3-5-sonnet-20240620",
                adapter: AdapterKind::Anthropic,
            },
            Model::Claude3Haiku => Info {
                //
                name: "claude-3-haiku-20240307",
                adapter: AdapterKind::Anthropic,
            },
            Model::Claude3Opus => Info {
                //
                name: "claude-3-opus-20240229",
                adapter: AdapterKind::Anthropic,
            },
            Model::Claude3Sonnet => Info {
                //
                name: "claude-3-sonnet-20240229",
                adapter: AdapterKind::Anthropic,
            },
            Model::Command => Info {
                //
                name: "command",
                adapter: AdapterKind::Cohere,
            },
            Model::CommandLight => Info {
                //
                name: "command-light",
                adapter: AdapterKind::Cohere,
            },
            Model::CommandLightNightly => Info {
                //
                name: "command-light-nightly",
                adapter: AdapterKind::Cohere,
            },
            Model::CommandNightly => Info {
                //
                name: "command-nightly",
                adapter: AdapterKind::Cohere,
            },
            Model::CommandR => Info {
                //
                name: "command-r",
                adapter: AdapterKind::Cohere,
            },
            Model::CommandRPlus => Info {
                //
                name: "command-r-plus",
                adapter: AdapterKind::Cohere,
            },
            Model::Gemini10Pro => Info {
                //
                name: "gemini-1.0-pro",
                adapter: AdapterKind::Gemini,
            },
            Model::Gemini15Flash => Info {
                //
                name: "gemini-1.5-flash",
                adapter: AdapterKind::Gemini,
            },
            Model::Gemini15FlashLatest => Info {
                //
                name: "gemini-1.5-flash-latest",
                adapter: AdapterKind::Gemini,
            },
            Model::Gemini15Pro => Info {
                //
                name: "gemini-1.5-pro",
                adapter: AdapterKind::Gemini,
            },
            Model::Gemma2b => Info {
                //
                name: "gemma:2b",
                adapter: AdapterKind::Ollama,
            },
            Model::Gemma7b => Info {
                //
                name: "gemma-7b-it",
                adapter: AdapterKind::Groq,
            },
            Model::Gemma9b => Info {
                //
                name: "gemma2-9b-it",
                adapter: AdapterKind::Groq,
            },
            Model::Gpt35Turbo => Info {
                //
                name: "gp-3.5-turbo",
                adapter: AdapterKind::OpenAI,
            },
            Model::Gpt4 => Info {
                //
                name: "gpt-4",
                adapter: AdapterKind::OpenAI,
            },
            Model::Gpt4o => Info {
                //
                name: "gpt-4o",
                adapter: AdapterKind::OpenAI,
            },
            Model::Gpt4oMini => Info {
                //
                name: "gpt-4o-mini",
                adapter: AdapterKind::OpenAI,
            },
            Model::Gpt4Turbo => Info {
                //
                name: "gpt-4-turbo",
                adapter: AdapterKind::OpenAI,
            },
            Model::Llama405bReasoning => Info {
                //
                name: "llama-3.1-405b-reasoning",
                adapter: AdapterKind::Groq,
            },
            Model::Llama70b => Info {
                //
                name: "llama3-70b-8192",
                adapter: AdapterKind::Groq,
            },
            Model::Llama70bVersatile => Info {
                //
                name: "llama-3.1-70b-versatile",
                adapter: AdapterKind::Groq,
            },
            Model::Llama8b => Info {
                //
                name: "llama3-8b-8192",
                adapter: AdapterKind::Groq,
            },
            Model::Llama8bInstant => Info {
                //
                name: "llama-3.1-8b-instant",
                adapter: AdapterKind::Groq,
            },

            Model::LlamaGroq70bToolUsePreview => Info {
                //
                name: "llama3-groq-70b-8192-tool-use-preview",
                adapter: AdapterKind::Groq,
            },
            Model::LlamaGroq8b => Info {
                //
                name: "llama3-groq-8b-8192-tool-use-preview",
                adapter: AdapterKind::Groq,
            },
            Model::Mixtral7b => Info {
                //
                name: "mixtral-8x7b-32768",
                adapter: AdapterKind::Groq,
            },
        }
    }

    pub fn name(&self) -> &'static str {
        self.info().name
    }

    pub fn adapter(&self) -> AdapterKind {
        self.info().adapter
    }
}

#[derive(Clone)]
struct Info {
    pub name: &'static str,
    pub adapter: AdapterKind,
}
