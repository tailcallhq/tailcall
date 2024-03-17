#[derive(Clone, Debug, strum_macros::Display)]
pub enum Object {
    Path(Vec<String>),
}
