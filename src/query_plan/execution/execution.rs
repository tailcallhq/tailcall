use std::fmt::{Display, Write};

use indenter::indented;

use super::super::resolver::Id;

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
                    ExecutionStep::Resolve(_) => unreachable!(),
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
    pub fn parallel(mut steps: Vec<ExecutionStep>) -> Self {
        if steps.len() == 1 {
            steps.pop().unwrap()
        } else {
            ExecutionStep::Parallel(steps)
        }
    }

    pub fn sequential(mut steps: Vec<ExecutionStep>) -> Self {
        if steps.len() == 1 {
            steps.pop().unwrap()
        } else {
            ExecutionStep::Sequential(steps)
        }
    }
}
