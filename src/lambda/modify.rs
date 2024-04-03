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
                    Expression::Context(_) => expr,
                    Expression::Literal(_) => expr,
                    Expression::EqualTo(expr1, expr2) => {
                        Expression::EqualTo(expr1.modify_box(modifier), expr2.modify_box(modifier))
                    }
                    Expression::IO(_) => expr,
                    Expression::Cache(Cache { expr, max_age }) => {
                        Expression::Cache(Cache { expr: expr.modify_box(modifier), max_age })
                    }
                    Expression::Input(expr, path) => {
                        Expression::Input(expr.modify_box(modifier), path)
                    }

                    Expression::Concurrency(conc, expr) => {
                        Expression::Concurrency(conc, expr.modify_box(modifier))
                    }
                    Expression::Protect(expr) => Expression::Protect(expr.modify_box(modifier)),
                }
            }
        }
    }
}
