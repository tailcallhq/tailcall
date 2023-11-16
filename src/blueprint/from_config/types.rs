use crate::config::Config;
use crate::try_fold::TryFold;

pub type TryFoldConfig<'a, A> = TryFold<'a, Config, A, String>;
