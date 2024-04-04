use anyhow::Result;
use async_graphql::Value;
use futures_util::future::join_all;
use indexmap::IndexMap;

use crate::{
    http::RequestContext,
    lambda::{Concurrent, Eval, EvaluationContext, ResolverContextLike},
};

use super::{
    plan::{ExecutionPlan, Fields, GeneralPlan},
    resolver::Id,
};

pub trait PlanExecutor {
    fn resolved_value(&mut self, id: &Id) -> Option<Result<Value>>;
}

pub struct SimpleExecutor<'a> {
    general_plan: &'a GeneralPlan,
    execution_plan: &'a ExecutionPlan<'a>,
    resolved: IndexMap<Id, Result<Value>>,
}

impl<'a> SimpleExecutor<'a> {
    pub fn new(general_plan: &'a GeneralPlan, execution_plan: &'a ExecutionPlan<'a>) -> Self {
        Self { general_plan, execution_plan, resolved: IndexMap::default() }
    }

    #[async_recursion::async_recursion]
    async fn inner_resolve<Ctx: ResolverContextLike<'a> + Sync + Send>(
        &mut self,
        req_ctx: &'a RequestContext,
        graphql_ctx: &'a Ctx,
        fields: &Fields,
    ) {
        if let Fields::Complex { children, .. } = fields {
            let plans = children.values().filter_map(|fields| {
                fields
                    .field_plan_id()
                    .and_then(|id| self.general_plan.field_plans.get(*id))
            });

            let results = join_all(plans.clone().map(|field_plan| {
                let eval_ctx = EvaluationContext::new(req_ctx, graphql_ctx);

                field_plan.eval(eval_ctx)
            }))
            .await;

            for (field_plan, value) in plans.zip(results) {
                self.resolved.insert(field_plan.id, value);
            }

            for fields in children.values() {
                self.inner_resolve(req_ctx, graphql_ctx, fields).await;
            }
        }
    }

    pub async fn resolve<Ctx: ResolverContextLike<'a> + Sync + Send>(
        &mut self,
        req_ctx: &'a RequestContext,
        graphql_ctx: &'a Ctx,
    ) {
        self.inner_resolve(req_ctx, graphql_ctx, &self.execution_plan.fields)
            .await
    }
}

impl<'a> PlanExecutor for SimpleExecutor<'a> {
    fn resolved_value(&mut self, id: &Id) -> Option<Result<Value>> {
        self.resolved.swap_remove(id)
    }
}
