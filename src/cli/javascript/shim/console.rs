use mini_v8::MiniV8;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{javascript::serde_v8::SerdeV8, CLIError},
    ToAnyHow,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JSError {
    message: String,
    stack: String,
}

impl From<JSError> for CLIError {
    fn from(value: JSError) -> Self {
        CLIError::new(&value.message).trace(value.stack.split('\n').map(|a| a.to_owned()).collect())
    }
}

pub fn init(v8: &MiniV8) -> anyhow::Result<()> {
    let console = v8.create_object();
    console
        .set("log", v8.create_function(console_log))
        .or_anyhow("could not set console.log: ")?;

    console
        .set("error", v8.create_function(console_err))
        .or_anyhow("could not set console.error: ")?;

    v8.global()
        .set("console", console)
        .or_anyhow("could not set global.console: ")?;
    Ok(())
}

fn console_log(invocation: mini_v8::Invocation) -> Result<mini_v8::Value, mini_v8::Error> {
    let args = get_console_message(invocation);
    println!("{}", args);
    Ok(mini_v8::Value::Undefined)
}

fn console_err(invocation: mini_v8::Invocation) -> Result<mini_v8::Value, mini_v8::Error> {
    let err = get_console_error(invocation);
    eprintln!("{}", err.to_string());
    Ok(mini_v8::Value::Undefined)
}

fn get_console_message(invocation: mini_v8::Invocation) -> String {
    invocation
        .args
        .iter()
        .flat_map(|v| {
            let p = serde_json::Value::from_v8(v).or_anyhow("could not get console message");
            Some(p.ok()?.to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn get_console_error(invocation: mini_v8::Invocation) -> CLIError {
    let mut error = CLIError::new("Javascript error").color(true);
    if let Some(value) = invocation.args.iter().next() {
        if let Ok(inner) = JSError::from_v8(value) {
            error = error.caused_by(vec![inner.into()])
        }
    }
    error
}
