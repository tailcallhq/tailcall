use tailcall_valid::{Valid, ValidationError, Validator};

use super::BlueprintError;
use crate::core::config::{Link, LinkType};
use crate::core::directive::DirectiveCodec;

pub struct Links;

impl TryFrom<Vec<Link>> for Links {
    type Error = ValidationError<crate::core::blueprint::BlueprintError>;

    fn try_from(links: Vec<Link>) -> Result<Self, Self::Error> {
        Valid::from_iter(links.iter().enumerate(), |(pos, link)| {
            Valid::succeed(link.to_owned())
                .and_then(|link| {
                    if link.src.is_empty() {
                        Valid::fail(BlueprintError::LinkSrcCannotBeEmpty)
                    } else {
                        Valid::succeed(link)
                    }
                })
                .and_then(|link| {
                    if let Some(id) = &link.id {
                        if links.iter().filter(|l| l.id.as_ref() == Some(id)).count() > 1 {
                            return Valid::fail(BlueprintError::Duplicated(id.clone()));
                        }
                    }
                    Valid::succeed(link)
                })
                .trace(&pos.to_string())
        })
        .and_then(|links| {
            let script_links = links
                .iter()
                .filter(|l| l.type_of == LinkType::Script)
                .collect::<Vec<&Link>>();

            if script_links.len() > 1 {
                Valid::fail(BlueprintError::OnlyOneScriptLinkAllowed)
            } else {
                Valid::succeed(links)
            }
        })
        .and_then(|links| {
            let key_links = links
                .iter()
                .filter(|l| l.type_of == LinkType::Key)
                .collect::<Vec<&Link>>();

            if key_links.len() > 1 {
                Valid::fail(BlueprintError::OnlyOneKeyLinkAllowed)
            } else {
                Valid::succeed(links)
            }
        })
        .trace(Link::trace_name().as_str())
        .trace("schema")
        .map_to(Links)
        .to_result()
    }
}
