pub trait CacheKey<Ctx> {
    fn cache_key(&self, ctx: &Ctx) -> anyhow::Result<u64>;
}
