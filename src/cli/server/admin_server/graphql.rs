use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct Config {
    pub sdl: String,
}

#[derive(SimpleObject)]
pub struct Query {
    pub config: Config,
}
