use std::{convert::Infallible, marker::PhantomData};

use tailcall_valid::Valid;

use crate::core::{
    counter::{Count, Counter},
    ir::model::{IrId, IR},
    jit::{Field, OperationPlan},
    Transform,
};

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

impl<A> WrapDefer<A> {
    pub fn new() -> Self {
        Self { _marker: PhantomData, defer_id: Counter::new(0) }
    }
    /// goes through selection and finds out IR's that needs to be deferred.
    #[inline]
    fn detect_and_wrap(&self, field: &mut Field<A>, path: &mut Vec<String>) {
        path.push(field.output_name.clone());
        for selection in field.selection.iter_mut() {
            if let Some(ir) = std::mem::take(&mut selection.ir) {
                let ir = if selection
                    .directives
                    .iter()
                    .find(|d| d.name == "defer")
                    .is_some()
                    && !check_dependent_irs(&ir)
                {
                    IR::Deferred {
                        ir: Box::new(ir),
                        path: path.clone(),
                        id: IrId::new(self.defer_id.next()),
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

impl<A> Transform for WrapDefer<A> {
    type Value = OperationPlan<A>;
    type Error = Infallible;
    fn transform(&self, mut plan: Self::Value) -> Valid<Self::Value, Self::Error> {
        plan.selection.iter_mut().for_each(|f| {
            if let Some(ir) = std::mem::take(&mut f.ir) {
                let ir = if f.directives.iter().find(|d| d.name == "defer").is_some()
                    && !check_dependent_irs(&ir)
                {
                    IR::Deferred {
                        ir: Box::new(ir),
                        path: vec![],
                        id: IrId::new(self.defer_id.next()),
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
