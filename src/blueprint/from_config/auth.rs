use std::sync::Arc;

use crate::auth::context::GlobalAuthContext;
use crate::config::{Auth, Upstream};
use crate::directive::DirectiveCodec;
use crate::http::DefaultHttpClient;
use crate::valid::Valid;

pub fn to_auth(auth: &Auth) -> Valid<Auth, String> {
  GlobalAuthContext::new(auth, Arc::new(DefaultHttpClient::new(&Upstream::default())))
    .map_to(auth.clone())
    .trace(Auth::trace_name().as_str())
}
