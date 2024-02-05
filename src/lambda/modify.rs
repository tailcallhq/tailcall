use super::{Cached, Expression};

impl Expression {
    fn modify_box<F>(self, f: F) -> Box<Expression>
    where
        F: Fn(&Expression) -> Option<Expression>,
    {
        Box::new(self.modify(f))
    }

    pub fn modify<F>(self, f: F) -> Expression
    where
        F: Fn(&Expression) -> Option<Expression>,
    {
        let modified = f(&self);
        match modified {
            Some(expr) => expr,
            None => {
                let expr = self;
                match expr {
                    Expression::Context(_) => expr,
                    Expression::Literal(_) => expr,
                    Expression::EqualTo(expr1, expr2) => {
                        Expression::EqualTo(expr1.modify_box(&f), expr2.modify_box(&f))
                    }
                    Expression::IO(_) => expr,
                    Expression::Cached(_) => expr,
                    Expression::Input(_, _) => expr,
                    Expression::Logic(expr) => match expr {
                        super::Logic::If { cond, then, els } => {
                            Expression::Logic(super::Logic::If {
                                cond: cond.modify_box(&f),
                                then: then.modify_box(&f),
                                els: els.modify_box(&f),
                            })
                        }
                        super::Logic::And(exprs) => Expression::Logic(super::Logic::And(
                            exprs.into_iter().map(|expr| expr.modify(&f)).collect(),
                        )),
                        super::Logic::Or(exprs) => Expression::Logic(super::Logic::Or(
                            exprs.into_iter().map(|expr| expr.modify(&f)).collect(),
                        )),
                        super::Logic::Cond(branches) => Expression::Logic(super::Logic::Cond(
                            branches
                                .into_iter()
                                .map(|(cond, expr)| (cond.modify_box(&f), expr.modify_box(&f)))
                                .collect(),
                        )),
                        super::Logic::DefaultTo(expr1, expr2) => Expression::Logic(
                            super::Logic::DefaultTo(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Logic::IsEmpty(expr) => {
                            Expression::Logic(super::Logic::IsEmpty(expr.modify_box(&f)))
                        }
                        super::Logic::Not(expr) => {
                            Expression::Logic(super::Logic::Not(expr.modify_box(&f)))
                        }
                    },
                    Expression::Relation(expr) => match expr {
                        super::Relation::Intersection(expr1, expr2) => {
                            Expression::Relation(super::Relation::Intersection(
                                expr1.modify_box(&f),
                                expr2.modify_box(&f),
                            ))
                        }
                        super::Relation::Difference(expr1, expr2) => Expression::Relation(
                            super::Relation::Difference(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Equals(expr1, expr2) => Expression::Relation(
                            super::Relation::Equals(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Gt(expr1, expr2) => Expression::Relation(
                            super::Relation::Gt(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Gte(expr1, expr2) => Expression::Relation(
                            super::Relation::Gte(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Lt(expr1, expr2) => Expression::Relation(
                            super::Relation::Lt(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Lte(expr1, expr2) => Expression::Relation(
                            super::Relation::Lte(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::Max(expr) => {
                            Expression::Relation(super::Relation::Max(expr.modify_box(&f)))
                        }
                        super::Relation::Min(expr) => {
                            Expression::Relation(super::Relation::Min(expr.modify_box(&f)))
                        }
                        super::Relation::PathEq(expr1, expr2, expr3) => {
                            Expression::Relation(super::Relation::PathEq(
                                expr1.modify_box(&f),
                                expr2.modify_box(&f),
                                expr3.modify_box(&f),
                            ))
                        }
                        super::Relation::PropEq(expr1, expr2, expr3) => {
                            Expression::Relation(super::Relation::PropEq(
                                expr1.modify_box(&f),
                                expr2.modify_box(&f),
                                expr3.modify_box(&f),
                            ))
                        }
                        super::Relation::SortPath(expr1, expr2) => Expression::Relation(
                            super::Relation::SortPath(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Relation::SymmetricDifference(expr1, expr2) => {
                            Expression::Relation(super::Relation::SymmetricDifference(
                                expr1.modify_box(&f),
                                expr2.modify_box(&f),
                            ))
                        }
                        super::Relation::Union(expr1, expr2) => Expression::Relation(
                            super::Relation::Union(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                    },
                    Expression::List(expr) => match expr {
                        super::List::Concat(_) => todo!(),
                    },
                    Expression::Concurrency(conc, expr) => {
                        Expression::Concurrency(conc, expr.modify_box(&f))
                    }
                    Expression::Math(expr) => match expr {
                        super::Math::Mod(expr1, expr2) => Expression::Math(super::Math::Mod(
                            expr1.modify_box(&f),
                            expr2.modify_box(&f),
                        )),
                        super::Math::Add(expr1, expr2) => Expression::Math(super::Math::Add(
                            expr1.modify_box(&f),
                            expr2.modify_box(&f),
                        )),
                        super::Math::Dec(expr) => {
                            Expression::Math(super::Math::Dec(expr.modify_box(&f)))
                        }
                        super::Math::Divide(expr1, expr2) => Expression::Math(super::Math::Divide(
                            expr1.modify_box(&f),
                            expr2.modify_box(&f),
                        )),
                        super::Math::Inc(expr) => {
                            Expression::Math(super::Math::Inc(expr.modify_box(&f)))
                        }
                        super::Math::Multiply(expr1, expr2) => Expression::Math(
                            super::Math::Multiply(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Math::Negate(expr) => {
                            Expression::Math(super::Math::Negate(expr.modify_box(&f)))
                        }
                        super::Math::Product(expr) => Expression::Math(super::Math::Product(
                            expr.into_iter().map(|expr| expr.modify(&f)).collect(),
                        )),
                        super::Math::Subtract(expr1, expr2) => Expression::Math(
                            super::Math::Subtract(expr1.modify_box(&f), expr2.modify_box(&f)),
                        ),
                        super::Math::Sum(expr) => {
                            expr.into_iter().map(|expr| expr.modify(&f)).collect()
                        }
                    },
                    Expression::Concurrency(conc, expr) => {
                        Expression::Concurrency(conc, expr.modify_box(&f))
                    }
                }
            }
        }
    }
}
