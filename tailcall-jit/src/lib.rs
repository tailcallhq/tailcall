use std::collections::HashMap;

type Id = u64;

#[derive(Clone)]
struct Node<A> {
    parent_id: Option<Id>,
    task: A,
}

#[derive(Clone)]
struct ExecutionPlan<'a, A> {
    nodes: HashMap<Id, Node<&'a A>>,
}

impl<'a, A> ExecutionPlan<'a, A> {
    fn get(&self, id: &Id) -> Option<&Node<&A>> {
        self.nodes.get(&id)
    }

    /// Swap all children from `from` to `to`
    fn swap_children(mut self, from: &Id, to: &Id) -> Self {
        if from == to {
            return self;
        }

        for node in self.nodes.values_mut() {
            if node.parent_id == Some(*from) {
                node.parent_id = Some(*to);
            }
        }
        self
    }

    /// Removes a node from the plan
    fn remove(mut self, id: &Id) -> Self {
        self.nodes.remove(&id);
        self
    }

    /// Find all nodes with the same task
    fn duplicates(&self) -> Vec<(Id, Id)>
    where
        A: Task,
    {
        self.nodes
            .iter()
            .filter_map(|(&from_id, to)| {
                self.nodes
                    .iter()
                    .find(|(&to_id, &ref from)| from_id != to_id && to.task == from.task)
                    .map(|(&to, _)| (from_id, to))
            })
            .collect()
    }

    /// Checks if the plan exists
    fn contains(&self, id: &Id) -> bool {
        self.nodes.contains_key(id)
    }

    /// Find all plans that have no parent
    fn dangling_plans(&self) -> Vec<Id> {
        self.nodes
            .iter()
            .filter(|(_, node)| {
                if let Some(parent_id) = node.parent_id {
                    !self.contains(&parent_id)
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Find all plans that don't depend on their parent plans
    fn independent_plans(&self) -> Vec<Id>
    where
        A: Task,
    {
        self.nodes
            .iter()
            .filter(|(_, node)| {
                if let Some(parent_id) = node.parent_id {
                    if let Some(parent) = self.get(&parent_id) {
                        !node.task.depends_on(parent.task)
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

trait Task: Eq {
    fn depends_on(&self, other: &Self) -> bool;
}

trait Transformer<A> {
    fn transform<'a>(&'a self, plan: ExecutionPlan<'a, A>) -> ExecutionPlan<'a, A>;
}

struct Dedupe {}

impl<A: Task> Transformer<A> for Dedupe {
    fn transform<'a>(&'a self, mut plan: ExecutionPlan<'a, A>) -> ExecutionPlan<'a, A> {
        let duplicates: Vec<(Id, Id)> = plan.duplicates();

        for (from, to) in duplicates {
            plan = plan.swap_children(&from, &to).remove(&from);
        }

        plan
    }
}

struct TreeShake {
    max_count: u64,
}

impl<A> Transformer<A> for TreeShake {
    fn transform<'a>(&'a self, mut plan: ExecutionPlan<'a, A>) -> ExecutionPlan<'a, A> {
        let mut changes = true;
        let mut count = 0;
        while changes && count < self.max_count {
            changes = false;
            count += 1;
            let dangling_plans = plan.dangling_plans();
            for dangling in dangling_plans {
                plan = plan.remove(&dangling);
                changes = true;
            }
        }

        plan
    }
}

struct ShiftRoot {}

impl<A: Task> Transformer<A> for ShiftRoot {
    fn transform<'a>(&'a self, mut plan: ExecutionPlan<'a, A>) -> ExecutionPlan<'a, A> {
        let independent_plans = plan.independent_plans();
        for id in independent_plans {
            if let Some(node) = plan.nodes.get_mut(&id) {
                node.parent_id = None;
            }
        }
        plan
    }
}
