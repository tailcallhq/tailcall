use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Adapter {
    Groq(GroqModel),
}

#[derive(Clone)]
pub enum GroqModel {
    Llama38b8192,
}

impl Display for Adapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Adapter::Groq(g) => g.to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Display for GroqModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GroqModel::Llama38b8192 => "Llama3_8b_8192",
        };
        write!(f, "{}", str)
    }
}
