use std::{convert::Infallible, marker::PhantomData};

use tailcall_valid::Valid;

use crate::core::{
    counter::{Count, Counter}, ir::model::{IrId, IR}, jit::{Field, OperationPlan}, Transform
};

pub struct WrapDefer<A>{
    _marker: PhantomData<A>,
    defer_id: Counter<usize>,
}

impl<A> WrapDefer<A> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            defer_id: Counter::new(0),
        }
    }
    /// goes through selection and finds out IR's that needs to be deferred.
    #[inline]
    fn detect_and_wrap(&self,field: &mut Field<A>, path: &mut Vec<String>) {
        path.push(field.output_name.clone());
        for selection in field.selection.iter_mut() {
            if let Some(ir) = std::mem::take(&mut selection.ir) {
                let ir = if selection
                    .directives
                    .iter()
                    .find(|d| d.name == "defer")
                    .is_some()
                    && field.ir.is_some()
                {
                    IR::Deferred { ir: Box::new(ir), path: vec![], id: IrId::new(self.defer_id.next()) }
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
        plan.selection
            .iter_mut()
            .for_each(|f| self.detect_and_wrap(f, &mut vec![]));
        Valid::succeed(plan)
    }
}
