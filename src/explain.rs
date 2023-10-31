use unleash_yggdrasil::state::EnrichedContext;

use crate::compile::{ExecutionNode, NodeType};

trait RepresentDisabled {
    fn as_disabled(&self) -> &str;
}

impl RepresentDisabled for bool {
    fn as_disabled(&self) -> &str {
        if *self {
            "enabled"
        } else {
            "disabled"
        }
    }
}

pub(crate) trait Executable {
    fn execute(&self, context: &EnrichedContext) -> ExecutionResult;
}

impl Executable for ExecutionNode {
    fn execute(&self, context: &EnrichedContext) -> ExecutionResult {
        let result = (self.compiled_rule)(context);
        let children_result: Vec<ExecutionResult> = self
            .children
            .iter()
            .map(|child| child.execute(context))
            .collect();

        let node_type = self.metadata.node_type.clone();
        let child_state_iterator = children_result
            .iter()
            .map(|child| child.node_enabled && child.children_enabled.unwrap_or(true));

        let children_enabled = match node_type {
            NodeType::Constraint => None,
            NodeType::Strategy => Some(child_state_iterator.clone().all(|state| state)),
            NodeType::Toggle => Some(child_state_iterator.clone().any(|state| state)),
        };

        ExecutionResult {
            node_enabled: result,
            children_enabled,
            children_result,
            node_type,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExecutionResult {
    node_enabled: bool,
    children_enabled: Option<bool>,
    children_result: Vec<ExecutionResult>,
    node_type: NodeType,
}

impl std::fmt::Debug for ExecutionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut children = String::new();
        for child in &self.children {
            children.push_str(&format!("{:#?}\n", child));
        }

        write!(
            f,
            "ExecutionNode {{ rule: {}, children: {} }}",
            self.rule, children
        )
    }
}
