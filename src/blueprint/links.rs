use crate::config::Link;
use crate::directive::DirectiveCodec;
use crate::valid::{Valid, ValidationError, Validator};

// Wrap the Vec<Link> is necessary to implement TryFrom
#[derive(Clone, Debug)]
pub struct Links(Vec<Link>);

impl TryFrom<Vec<Link>> for Links {
    type Error = ValidationError<String>;

    fn try_from(links: Vec<Link>) -> Result<Self, Self::Error> {
        Valid::from_iter(links.iter().enumerate(), |(pos, link)| {
            Valid::succeed(link.to_owned())
                .and_then(|link| {
                    if link.src.is_empty() {
                        Valid::fail("Link src cannot be empty".to_string())
                    } else {
                        Valid::succeed(link)
                    }
                })
                .and_then(|link| {
                    if let Some(id) = &link.id {
                        if links.iter().filter(|l| l.id.as_ref() == Some(id)).count() > 1 {
                            return Valid::fail(format!("Duplicated id: {}", id));
                        }
                    }
                    Valid::succeed(link)
                })
                .trace(&pos.to_string())
                .trace(Link::trace_name().as_str())
                .trace("schema")
        })
        .to_result()
        .map(Links)
    }
}
