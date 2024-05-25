use std::collections::HashMap;

type Id = u64;

#[derive(Clone)]
struct Node<A> {
    parent_id: Option<Id>,
    task: A,
}

/// Representation of the actual execution plan.
/// Internally it represents a graph of nodes where each node has an Id of its
/// own and refers to a parent node to maintain the dependency relationship.
#[derive(Clone)]
struct ExecutionGraph<'a, A>(HashMap<Id, Node<&'a A>>);

impl<'a, A> ExecutionGraph<'a, A> {
    fn get(&self, id: &Id) -> Option<&Node<&A>> {
        self.0.get(&id)
    }

    /// Swap all children of a node with the id=`from` and move it's children to
    /// the node with the id=`to`
    fn swap_children(mut self, from: &Id, to: &Id) -> Self {
        if from == to {
            return self;
        }

        for node in self.0.values_mut() {
            if node.parent_id == Some(*from) {
                node.parent_id = Some(*to);
            }
        }
        self
    }

    /// Removes a node from the plan
    fn remove(mut self, id: &Id) -> Self {
        self.0.remove(&id);
        self
    }

    /// Checks if the plan exists
    fn contains(&self, id: &Id) -> bool {
        self.0.contains_key(id)
    }

    /// Find all plans that have no parent
    fn dangling_plans(&self) -> Vec<Id> {
        self.0
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
        self.0
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

struct DuplicateTasks<A>(Vec<A>);

trait Task: Eq {
    fn depends_on(&self, other: &Self) -> bool;
}

trait Transformer<A> {
    fn transform<'a>(&'a self, plan: ExecutionGraph<'a, A>) -> ExecutionGraph<'a, A>;
}

/// Takes all the tasks that are equal and merges them into a new Task.
/// The new task contains children from the
struct Dedupe {}

impl<A: Task> Transformer<A> for Dedupe {
    fn transform<'a>(&'a self, plan: ExecutionGraph<'a, A>) -> ExecutionGraph<'a, A> {
        todo!()
    }
}

/// Drops all the nodes that who's parent Ids don't exist.
struct TreeShake {
    max_count: u64,
}

impl<A> Transformer<A> for TreeShake {
    fn transform<'a>(&'a self, mut plan: ExecutionGraph<'a, A>) -> ExecutionGraph<'a, A> {
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

/// Performs a special check to see if the task truly depends on the parent task
/// by calling tha `Task::depends_on` function. If the task doesn't depend on
/// the parent then it resets the parent_id to none, effectively moving up the
/// execution plan.
struct ShiftToRoot {}

impl<A: Task> Transformer<A> for ShiftToRoot {
    fn transform<'a>(&'a self, mut plan: ExecutionGraph<'a, A>) -> ExecutionGraph<'a, A> {
        let independent_plans = plan.independent_plans();
        for id in independent_plans {
            if let Some(node) = plan.nodes.get_mut(&id) {
                node.parent_id = None;
            }
        }
        plan
    }
}
