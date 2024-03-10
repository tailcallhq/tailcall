use std::str::FromStr;
use std::{env, fmt};

use colored::Colorize;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
struct FmtLevel<'a> {
    level: &'a Level,
    ansi: bool,
}

impl<'a> FmtLevel<'a> {
    pub(crate) fn new(level: &'a Level, ansi: bool) -> Self {
        Self { level, ansi }
    }
}

const TRACE_STR: &str = "TRACE";
const DEBUG_STR: &str = "DEBUG";
const INFO_STR: &str = "INFO";
const WARN_STR: &str = "WARN";
const ERROR_STR: &str = "ERROR";

impl<'a> fmt::Display for FmtLevel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ansi {
            match *self.level {
                Level::TRACE => write!(f, "{}", TRACE_STR.magenta()),
                Level::DEBUG => write!(f, "{}", DEBUG_STR.blue()),
                Level::INFO => write!(f, "{}", INFO_STR.green()),
                Level::WARN => write!(f, "{}", WARN_STR.yellow()),
                Level::ERROR => write!(f, "{}", ERROR_STR.red()),
            }
        } else {
            match *self.level {
                Level::TRACE => f.pad(TRACE_STR),
                Level::DEBUG => f.pad(DEBUG_STR),
                Level::INFO => f.pad(INFO_STR),
                Level::WARN => f.pad(WARN_STR),
                Level::ERROR => f.pad(ERROR_STR),
            }
        }
    }
}

struct CliFmt;

impl<S, N> FormatEvent<S, N> for CliFmt
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let fmt_level = FmtLevel::new(meta.level(), writer.has_ansi_escapes());
        write!(writer, "[{}] ", fmt_level)?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub fn default_tailcall_tracing() -> impl Subscriber {
    default_tracing().with(tailcall_filter_target())
}

pub fn default_crate_tracing(name: &'static str) -> impl Subscriber {
    default_tracing().with(filter_target(name))
}

pub fn default_tracing() -> impl Subscriber {
    const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_LOG_LEVEL";
    const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_LOG_LEVEL";

    let level = env::var(LONG_ENV_FILTER_VAR_NAME)
        .or(env::var(SHORT_ENV_FILTER_VAR_NAME))
        .ok()
        .and_then(|v| tracing::Level::from_str(&v).ok())
        // use the log level from the env if there is one, otherwise use the default.
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(level)
        .without_time()
        .with_target(false)
        .event_format(CliFmt)
        .finish()
}

pub fn tailcall_filter_target<S: Subscriber>() -> impl Layer<S> {
    filter_target("tailcall")
}

pub fn filter_target<S: Subscriber>(name: &'static str) -> impl Layer<S> {
    filter_fn(move |metadata| metadata.target().starts_with(name))
}
