use super::{Cache, Expression};

impl Expression {
    pub fn modify(self, mut f: impl FnMut(&Expression) -> Option<Expression>) -> Expression {
        self.modify_inner(&mut f)
    }

    fn modify_vec<F: FnMut(&Expression) -> Option<Expression>>(
        exprs: Vec<Self>,
        modifier: &mut F,
    ) -> Vec<Expression> {
        exprs
            .into_iter()
            .map(|expr| expr.modify_inner(modifier))
            .collect()
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
                    Expression::Cache(Cache { expr, .. }) => {
                        Expression::IO(expr).modify_inner(modifier)
                    }
                    Expression::Input(expr, path) => {
                        Expression::Input(expr.modify_box(modifier), path)
                    }
                    Expression::Logic(expr) => match expr {
                        super::Logic::If { cond, then, els } => {
                            Expression::Logic(super::Logic::If {
                                cond: cond.modify_box(modifier),
                                then: then.modify_box(modifier),
                                els: els.modify_box(modifier),
                            })
                        }
                        super::Logic::And(exprs) => {
                            Expression::Logic(super::Logic::And(Self::modify_vec(exprs, modifier)))
                        }
                        super::Logic::Or(exprs) => {
                            Expression::Logic(super::Logic::Or(Self::modify_vec(exprs, modifier)))
                        }
                        super::Logic::Cond(branches) => Expression::Logic(super::Logic::Cond(
                            branches
                                .into_iter()
                                .map(|(cond, expr)| {
                                    (cond.modify_box(modifier), expr.modify_box(modifier))
                                })
                                .collect(),
                        )),
                        super::Logic::DefaultTo(expr1, expr2) => {
                            Expression::Logic(super::Logic::DefaultTo(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Logic::IsEmpty(expr) => {
                            Expression::Logic(super::Logic::IsEmpty(expr.modify_box(modifier)))
                        }
                        super::Logic::Not(expr) => {
                            Expression::Logic(super::Logic::Not(expr.modify_box(modifier)))
                        }
                    },
                    Expression::Relation(expr) => match expr {
                        super::Relation::Intersection(exprs) => Expression::Relation(
                            super::Relation::Intersection(Self::modify_vec(exprs, modifier)),
                        ),
                        super::Relation::Difference(expr1, expr2) => {
                            Expression::Relation(super::Relation::Difference(
                                Self::modify_vec(expr1, modifier),
                                Self::modify_vec(expr2, modifier),
                            ))
                        }
                        super::Relation::Equals(expr1, expr2) => {
                            Expression::Relation(super::Relation::Equals(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::Gt(expr1, expr2) => {
                            Expression::Relation(super::Relation::Gt(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::Gte(expr1, expr2) => {
                            Expression::Relation(super::Relation::Gte(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::Lt(expr1, expr2) => {
                            Expression::Relation(super::Relation::Lt(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::Lte(expr1, expr2) => {
                            Expression::Relation(super::Relation::Lte(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::Max(exprs) => Expression::Relation(super::Relation::Max(
                            Self::modify_vec(exprs, modifier),
                        )),
                        super::Relation::Min(exprs) => Expression::Relation(super::Relation::Min(
                            Self::modify_vec(exprs, modifier),
                        )),
                        super::Relation::PathEq(expr1, path, expr2) => {
                            Expression::Relation(super::Relation::PathEq(
                                expr1.modify_box(modifier),
                                path,
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::PropEq(expr1, path, expr2) => {
                            Expression::Relation(super::Relation::PropEq(
                                expr1.modify_box(modifier),
                                path,
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Relation::SortPath(expr, path) => Expression::Relation(
                            super::Relation::SortPath(expr.modify_box(modifier), path),
                        ),
                        super::Relation::SymmetricDifference(expr1, expr2) => {
                            Expression::Relation(super::Relation::SymmetricDifference(
                                Self::modify_vec(expr1, modifier),
                                Self::modify_vec(expr2, modifier),
                            ))
                        }
                        super::Relation::Union(exprs1, exprs2) => {
                            Expression::Relation(super::Relation::Union(
                                Self::modify_vec(exprs1, modifier),
                                Self::modify_vec(exprs2, modifier),
                            ))
                        }
                    },
                    Expression::List(expr) => Expression::List(match expr {
                        super::List::Concat(exprs) => {
                            super::List::Concat(Self::modify_vec(exprs, modifier))
                        }
                    }),
                    Expression::Concurrency(conc, expr) => {
                        Expression::Concurrency(conc, expr.modify_box(modifier))
                    }
                    Expression::Math(expr) => match expr {
                        super::Math::Mod(expr1, expr2) => Expression::Math(super::Math::Mod(
                            expr1.modify_box(modifier),
                            expr2.modify_box(modifier),
                        )),
                        super::Math::Add(expr1, expr2) => Expression::Math(super::Math::Add(
                            expr1.modify_box(modifier),
                            expr2.modify_box(modifier),
                        )),
                        super::Math::Dec(expr) => {
                            Expression::Math(super::Math::Dec(expr.modify_box(modifier)))
                        }
                        super::Math::Divide(expr1, expr2) => Expression::Math(super::Math::Divide(
                            expr1.modify_box(modifier),
                            expr2.modify_box(modifier),
                        )),
                        super::Math::Inc(expr) => {
                            Expression::Math(super::Math::Inc(expr.modify_box(modifier)))
                        }
                        super::Math::Multiply(expr1, expr2) => {
                            Expression::Math(super::Math::Multiply(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Math::Negate(expr) => {
                            Expression::Math(super::Math::Negate(expr.modify_box(modifier)))
                        }
                        super::Math::Product(exprs) => Expression::Math(super::Math::Product(
                            Self::modify_vec(exprs, modifier),
                        )),
                        super::Math::Subtract(expr1, expr2) => {
                            Expression::Math(super::Math::Subtract(
                                expr1.modify_box(modifier),
                                expr2.modify_box(modifier),
                            ))
                        }
                        super::Math::Sum(exprs) => {
                            Expression::Math(super::Math::Sum(Self::modify_vec(exprs, modifier)))
                        }
                    },
                    Expression::Protected(expr) => Expression::Protected(expr.modify_box(modifier)),
                }
            }
        }
    }
}
