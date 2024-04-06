use std::{fmt::Display, sync::Mutex};

use anyhow::{anyhow, Result};
use async_graphql::Value;
use futures_util::future::join_all;
use indexmap::IndexMap;

use crate::{
    http::RequestContext,
    lambda::{EvaluationContext, ResolverContextLike},
    query_plan::{plan::GeneralPlan, resolver::Id},
};

use super::execution::ExecutionStep;

pub struct Executor<'a> {
    general_plan: &'a GeneralPlan,
}

pub struct ExecutionResult {
    resolved: IndexMap<Id, Result<Value>>,
}

struct ExecutorContext<'a, Ctx: ResolverContextLike<'a> + Sync + Send> {
    general_plan: &'a GeneralPlan,
    req_ctx: &'a RequestContext,
    graphql_ctx: &'a Ctx,
    resolved: Mutex<IndexMap<Id, Result<Value>>>,
}

impl Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.resolved)
    }
}

impl<'a> Executor<'a> {
    pub fn new(general_plan: &'a GeneralPlan) -> Self {
        Self { general_plan }
    }

    pub async fn execute<Ctx: ResolverContextLike<'a> + Sync + Send>(
        &self,
        req_ctx: &'a RequestContext,
        graphql_ctx: &'a Ctx,
        execution: &ExecutionStep,
    ) -> ExecutionResult {
        let executor_ctx = ExecutorContext {
            general_plan: self.general_plan,
            req_ctx,
            graphql_ctx,
            resolved: Mutex::new(IndexMap::new()),
        };

        executor_ctx.execute(execution).await;

        let resolved = executor_ctx.resolved.into_inner().unwrap();

        ExecutionResult { resolved }
    }
}

impl<'a, Ctx: ResolverContextLike<'a> + Sync + Send> ExecutorContext<'a, Ctx> {
    #[async_recursion::async_recursion]
    pub async fn execute(&self, execution: &ExecutionStep) {
        match execution {
            ExecutionStep::Resolve(id) => {
                let field_plan = self.general_plan.field_plans.get(**id);

                let result = if let Some(field_plan) = field_plan {
                    let eval_ctx = EvaluationContext::new(&self.req_ctx, self.graphql_ctx);

                    field_plan.eval(eval_ctx).await
                } else {
                    Err(anyhow!("Failed to resolve field_plan for id: {id}"))
                };

                self.resolved.lock().unwrap().insert(*id, result);
            }
            ExecutionStep::Sequential(steps) => {
                for step in steps {
                    self.execute(step).await;
                }
            }
            ExecutionStep::Parallel(steps) => {
                join_all(steps.iter().map(|step| self.execute(step))).await;
            }
        }
    }
}

impl ExecutionResult {
    pub fn resolved(&mut self, id: &Id) -> Option<Result<Value>> {
        self.resolved.swap_remove(id)
    }
}
