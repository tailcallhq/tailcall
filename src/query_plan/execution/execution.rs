use std::{
    fmt::{Display, Write},
    mem::{discriminant, Discriminant},
};

use indenter::indented;

use super::super::resolver::Id;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionStep {
    Resolve(Id),
    Sequential(Vec<ExecutionStep>),
    Parallel(Vec<ExecutionStep>),
}

impl Display for ExecutionStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStep::Resolve(id) => writeln!(f, "Resolve({id})"),
            ExecutionStep::Sequential(steps) | ExecutionStep::Parallel(steps) => {
                match &self {
                    ExecutionStep::Sequential(_) => writeln!(f, "Sequential:"),
                    ExecutionStep::Parallel(_) => writeln!(f, "Parallel:"),
                    _ => unreachable!(),
                }?;
                let f = &mut indented(f);

                for step in steps {
                    write!(f, "{}", step)?;
                }

                Ok(())
            }
        }
    }
}

impl ExecutionStep {
    fn inner_flatten(dscr: Discriminant<Self>, steps: Vec<Self>) -> Vec<Self> {
        let mut result = Vec::with_capacity(steps.len());

        for step in steps {
            let step = step.flatten();
            if dscr == discriminant(&step) {
                match step {
                    ExecutionStep::Sequential(sub_steps) | ExecutionStep::Parallel(sub_steps) => {
                        for sub_step in sub_steps {
                            result.push(sub_step);
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                if !step.is_empty() {
                    result.push(step);
                }
            }
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ExecutionStep::Sequential(steps) | ExecutionStep::Parallel(steps) => steps.is_empty(),
            _ => false,
        }
    }

    pub fn flatten(self) -> Self {
        let dscr = discriminant(&self);
        match self {
            ExecutionStep::Resolve(_) => self,
            ExecutionStep::Sequential(steps) => {
                let mut steps = Self::inner_flatten(dscr, steps);

                if steps.len() == 1 {
                    steps.pop().unwrap()
                } else {
                    ExecutionStep::Sequential(steps)
                }
            }
            ExecutionStep::Parallel(steps) => {
                let mut steps = Self::inner_flatten(dscr, steps);

                if steps.len() == 1 {
                    steps.pop().unwrap()
                } else {
                    ExecutionStep::Parallel(steps)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod flatten {
        use crate::query_plan::{execution::execution::ExecutionStep, resolver::Id};

        #[test]
        fn empty() {
            assert_eq!(
                ExecutionStep::Sequential(vec![]).flatten(),
                ExecutionStep::Sequential(vec![])
            );
            assert_eq!(
                ExecutionStep::Parallel(vec![]).flatten(),
                ExecutionStep::Parallel(vec![])
            );
        }

        #[test]
        fn single() {
            assert_eq!(
                ExecutionStep::Resolve(Id(0)).flatten(),
                ExecutionStep::Resolve(Id(0))
            );

            assert_eq!(
                ExecutionStep::Sequential(vec![ExecutionStep::Resolve(Id(0))]).flatten(),
                ExecutionStep::Resolve(Id(0))
            );

            assert_eq!(
                ExecutionStep::Parallel(vec![ExecutionStep::Resolve(Id(0))]).flatten(),
                ExecutionStep::Resolve(Id(0))
            );
        }

        #[test]
        fn sequential() {
            assert_eq!(
                ExecutionStep::Sequential(vec![
                    ExecutionStep::Resolve(Id(0)),
                    ExecutionStep::Sequential(vec![
                        ExecutionStep::Resolve(Id(1)),
                        ExecutionStep::Parallel(vec![]),
                        ExecutionStep::Resolve(Id(2)),
                    ]),
                    ExecutionStep::Resolve(Id(3)),
                    ExecutionStep::Parallel(vec![
                        ExecutionStep::Resolve(Id(4)),
                        ExecutionStep::Resolve(Id(5)),
                    ]),
                    ExecutionStep::Sequential(vec![ExecutionStep::Sequential(vec![
                        ExecutionStep::Resolve(Id(6)),
                        ExecutionStep::Resolve(Id(7))
                    ])])
                ])
                .flatten(),
                ExecutionStep::Sequential(vec![
                    ExecutionStep::Resolve(Id(0)),
                    ExecutionStep::Resolve(Id(1)),
                    ExecutionStep::Resolve(Id(2)),
                    ExecutionStep::Resolve(Id(3)),
                    ExecutionStep::Parallel(vec![
                        ExecutionStep::Resolve(Id(4)),
                        ExecutionStep::Resolve(Id(5)),
                    ]),
                    ExecutionStep::Resolve(Id(6)),
                    ExecutionStep::Resolve(Id(7))
                ])
            );
        }

        #[test]
        fn parallel() {
            assert_eq!(
                ExecutionStep::Parallel(vec![
                    ExecutionStep::Parallel(vec![
                        ExecutionStep::Resolve(Id(0)),
                        ExecutionStep::Resolve(Id(1)),
                        ExecutionStep::Resolve(Id(2)),
                    ]),
                    ExecutionStep::Resolve(Id(3)),
                    ExecutionStep::Sequential(vec![
                        ExecutionStep::Resolve(Id(4)),
                        ExecutionStep::Parallel(vec![
                            ExecutionStep::Resolve(Id(5)),
                            ExecutionStep::Resolve(Id(6)),
                        ]),
                        ExecutionStep::Resolve(Id(7))
                    ])
                ])
                .flatten(),
                ExecutionStep::Parallel(vec![
                    ExecutionStep::Resolve(Id(0)),
                    ExecutionStep::Resolve(Id(1)),
                    ExecutionStep::Resolve(Id(2)),
                    ExecutionStep::Resolve(Id(3)),
                    ExecutionStep::Sequential(vec![
                        ExecutionStep::Resolve(Id(4)),
                        ExecutionStep::Parallel(vec![
                            ExecutionStep::Resolve(Id(5)),
                            ExecutionStep::Resolve(Id(6)),
                        ]),
                        ExecutionStep::Resolve(Id(7))
                    ])
                ])
            )
        }
    }
}
