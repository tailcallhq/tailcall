use super::model::{Cache, Context, Map, IR};

impl IR {
    pub fn modify(self, mut f: impl FnMut(&IR) -> Option<IR>) -> IR {
        self.modify_inner(&mut f)
    }

    fn modify_box<F: FnMut(&IR) -> Option<IR>>(self, modifier: &mut F) -> Box<IR> {
        Box::new(self.modify_inner(modifier))
    }

    fn modify_inner<F: FnMut(&IR) -> Option<IR>>(self, modifier: &mut F) -> IR {
        let modified = modifier(&self);
        match modified {
            Some(expr) => expr,
            None => {
                let expr = self;
                match expr {
                    IR::Context(ctx) => match ctx {
                        Context::Value | Context::Path(_) => IR::Context(ctx),
                        Context::PushArgs { expr, and_then } => IR::Context(Context::PushArgs {
                            expr: expr.modify_box(modifier),
                            and_then: and_then.modify_box(modifier),
                        }),
                        Context::PushValue { expr, and_then } => IR::Context(Context::PushValue {
                            expr: expr.modify_box(modifier),
                            and_then: and_then.modify_box(modifier),
                        }),
                    },
                    IR::Dynamic(_) => expr,
                    IR::IO(_) => expr,
                    IR::Cache(Cache { io, max_age }) => {
                        let expr = *IR::IO(*io).modify_box(modifier);
                        match expr {
                            IR::IO(io) => IR::Cache(Cache { io: Box::new(io), max_age }),
                            expr => expr,
                        }
                    }
                    IR::Path(expr, path) => IR::Path(expr.modify_box(modifier), path),
                    IR::Protect(expr) => IR::Protect(expr.modify_box(modifier)),
                    IR::Map(Map { input, map }) => {
                        IR::Map(Map { input: input.modify_box(modifier), map })
                    }
                }
            }
        }
    }
}
