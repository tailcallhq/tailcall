/// The definitions transformer
pub(super) mod definitions;
pub(super) mod directive;
pub(super) mod enum_type;
/// Http transformer
pub(super) mod http;
pub(super) mod objects;
pub(super) mod scalar_type;
/// The schema transformer
pub(super) mod schema;
pub(super) mod server;
pub(super) mod union_type;
pub(super) mod update_const;
pub(super) mod update_group_by;
pub(super) mod update_inline;
pub(super) mod update_modify;
pub(super) mod update_unsafe;

use crate::valid::Valid as ValidDefault;

pub type Valid<A> = ValidDefault<A, String>;
