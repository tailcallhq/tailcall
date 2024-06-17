use derive_setters::Setters;
use value::ValueLike;

use super::*;
use crate::core::ir::model::*;
use crate::core::runtime::TargetRuntime;

/// NOTE: Cloning EvalContext should remain cheap for performance reasons.
#[allow(unused)]
#[derive(Clone, Setters)]
struct EvalContext<A> {
    #[setters(skip)]
    runtime: TargetRuntime,
    value: A,
    args: A,
    parent: A,
}

impl<A: ValueLike> EvalContext<A> {
    #[allow(unused)]
    pub fn new(runtime: TargetRuntime) -> Self {
        Self {
            runtime,
            value: A::default(),
            args: A::default(),
            parent: A::default(),
        }
    }

    #[allow(unused)]
    pub async fn eval(mut self, ir: IR) -> Result<A> {
        Box::pin(async move {
            match ir {
                IR::Context(ctx) => match ctx {
                    Context::Value => Ok(self.value.clone()),
                    Context::Path(path) => {
                        let a = path
                            .split_first()
                            .and_then(|(head, tail)| match head.as_str() {
                                "value" => self.value.path(tail),
                                "args" => self.args.path(tail),
                                "parent" => self.parent.path(tail),
                                _ => None,
                            });
                        Ok(a.unwrap_or(A::default()))
                    }
                    Context::PushArgs { expr, and_then } => {
                        let args = self.clone().eval(*expr).await?;
                        self.args(args).eval(*and_then).await
                    }
                    Context::PushValue { expr, and_then } => {
                        let value = self.clone().eval(*expr).await?;
                        self.value(value).eval(*and_then).await
                    }
                },
                IR::Dynamic(_) => todo!(),
                IR::IO(io) => match io {
                    IO::Http { req_template, group_by, dl_id, http_filter } => todo!(),
                    IO::GraphQL { req_template, field_name, batch, dl_id } => todo!(),
                    IO::Grpc { req_template, group_by, dl_id } => todo!(),
                    IO::Js { name } => todo!(),
                },
                IR::Cache(cache) => todo!(),
                IR::Path(_, _) => todo!(),
                IR::Protect(_) => todo!(),
                IR::Map(_) => todo!(),
            }
        })
        .await
    }
}
