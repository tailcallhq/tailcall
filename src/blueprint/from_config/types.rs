use crate::{try_fold::TryFold, config::Config};

pub type TryFoldConfig<'a, A> = TryFold<'a, Config, A, String>;
