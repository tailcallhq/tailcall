use std::convert::Infallible;
use std::fmt::Display;
use std::marker::PhantomData;

use tailcall_valid::Valid;

use crate::core::counter::{Count, Counter};
use crate::core::ir::model::{IrId, IR};
use crate::core::jit::{Field, OperationPlan};
use crate::core::Transform;

pub struct WrapDefer<A> {
    _marker: PhantomData<A>,
    defer_id: Counter<usize>,
}

fn check_dependent_irs(ir: &IR) -> bool {
    match ir {
        IR::IO(io) => io.is_dependent(),
        IR::Cache(cache) => cache.io.is_dependent(),
        IR::Deferred { .. } | IR::Service(_) | IR::ContextPath(_) | IR::Dynamic(_) => false,
        IR::Path(ir, _) => check_dependent_irs(ir),
        IR::Map(map) => check_dependent_irs(&map.input),
        IR::Pipe(l, r) => check_dependent_irs(l) || check_dependent_irs(r),
        IR::Discriminate(_, ir) => check_dependent_irs(ir),
        IR::Entity(map) => map.values().any(check_dependent_irs),
        IR::Protect(_, ir) => check_dependent_irs(ir),
    }
}

impl<A: Display> WrapDefer<A> {
    pub fn new() -> Self {
        Self { _marker: PhantomData, defer_id: Counter::new(0) }
    }
    /// goes through selection and finds out IR's that needs to be deferred.
    #[inline]
    fn detect_and_wrap(&self, field: &mut Field<A>, path: &mut Vec<String>) {
        path.push(field.output_name.clone());
        for selection in field.selection.iter_mut() {
            if let Some(ir) = std::mem::take(&mut selection.ir) {
                let ir =
                    if let Some(_defer) = selection.directives.iter().find(|d| d.name == "defer") {
                        let condition = _defer
                            .arguments
                            .iter()
                            .find(|(k, _)| k == "if")
                            .map(|(_, v)| v.to_string() == "true")
                            .unwrap_or(true);
                        let label = _defer
                            .arguments
                            .iter()
                            .find(|(k, _)| k == "label")
                            .map(|(_, v)| v.to_string());
                        if condition && !check_dependent_irs(&ir) {
                            IR::Deferred {
                                ir: Box::new(ir),
                                path: path.clone(),
                                id: IrId::new(self.defer_id.next()),
                                label,
                            }
                        } else {
                            ir
                        }
                    } else {
                        ir
                    };
                selection.ir = Some(ir);
            }

            self.detect_and_wrap(selection, path);
        }

        path.pop();
    }
}

impl<A: Display> Transform for WrapDefer<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;
    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        plan.selection.iter_mut().for_each(|f| {
            if let Some(ir) = std::mem::take(&mut f.ir) {
                let ir = if let Some(_defer) = f.directives.iter().find(|d| d.name == "defer") {
                    let condition = _defer
                        .arguments
                        .iter()
                        .find(|(k, _)| k == "if")
                        .map(|(_, v)| v.to_string() == "true")
                        .unwrap_or(true);   // defaults to true
                    let label = _defer
                        .arguments
                        .iter()
                        .find(|(k, _)| k == "label")
                        .map(|(_, v)| v.to_string());
                    if condition && !check_dependent_irs(&ir) {
                        IR::Deferred {
                            ir: Box::new(ir),
                            path: vec![],
                            id: IrId::new(self.defer_id.next()),
                            label,
                        }
                    } else {
                        ir
                    }
                } else {
                    ir
                };
                f.ir = Some(ir);
            }
            self.detect_and_wrap(f, &mut vec![])
        });
        Valid::succeed(plan)
    }
}
