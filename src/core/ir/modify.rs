use super::{Cache, IR};

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
                        super::Context::Value | super::Context::Path(_) => IR::Context(ctx),
                        super::Context::PushArgs { expr, and_then } => {
                            IR::Context(super::Context::PushArgs {
                                expr: expr.modify_box(modifier),
                                and_then: and_then.modify_box(modifier),
                            })
                        }
                        super::Context::PushValue { expr, and_then } => {
                            IR::Context(super::Context::PushValue {
                                expr: expr.modify_box(modifier),
                                and_then: and_then.modify_box(modifier),
                            })
                        }
                    },
                    IR::Dynamic(_) => expr,
                    IR::IO(_) => expr,
                    IR::Cache(Cache { expr, max_age }) => {
                        IR::Cache(Cache { expr: expr.modify_box(modifier), max_age })
                    }
                    IR::Path(expr, path) => IR::Path(expr.modify_box(modifier), path),
                    IR::Protect(expr) => IR::Protect(expr.modify_box(modifier)),
                }
            }
        }
    }
}
