use super::{Cache, Expression};

impl Expression {
    pub fn modify(self, mut f: impl FnMut(&Expression) -> Option<Expression>) -> Expression {
        self.modify_inner(&mut f)
    }

    fn modify_box<F: FnMut(&Expression) -> Option<Expression>>(
        self,
        modifier: &mut F,
    ) -> Box<Expression> {
        Box::new(self.modify_inner(modifier))
    }

    fn modify_inner<F: FnMut(&Expression) -> Option<Expression>>(
        self,
        modifier: &mut F,
    ) -> Expression {
        let modified = modifier(&self);
        match modified {
            Some(expr) => expr,
            None => {
                let expr = self;
                match expr {
                    Expression::Context(ctx) => match ctx {
                        super::Context::Value | super::Context::Path(_) => Expression::Context(ctx),
                        super::Context::PushArgs { expr, and_then } => {
                            Expression::Context(super::Context::PushArgs {
                                expr: expr.modify_box(modifier),
                                and_then: and_then.modify_box(modifier),
                            })
                        }
                        super::Context::PushValue { expr, and_then } => {
                            Expression::Context(super::Context::PushValue {
                                expr: expr.modify_box(modifier),
                                and_then: and_then.modify_box(modifier),
                            })
                        }
                    },
                    Expression::Jq(_) => expr,
                    Expression::Dynamic(_) => expr,
                    Expression::IO(_) => expr,
                    Expression::Cache(Cache { expr, max_age }) => {
                        Expression::Cache(Cache { expr: expr.modify_box(modifier), max_age })
                    }
                    Expression::Path(expr, path) => {
                        Expression::Path(expr.modify_box(modifier), path)
                    }
                    Expression::Protect(expr) => Expression::Protect(expr.modify_box(modifier)),
                }
            }
        }
    }
}
