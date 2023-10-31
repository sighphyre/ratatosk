use std::collections::HashMap;

use unleash_types::client_features::{ClientFeature, Constraint, Segment, Strategy};
use unleash_yggdrasil::{
    strategy_parsing::{compile_rule, RuleFragment},
    strategy_upgrade::{upgrade_constraint, upgrade_strategy},
};

#[derive(Debug, Clone)]
pub(crate) enum NodeType {
    Toggle,
    Strategy,
    Constraint,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeMetaData {
    pub(crate) node_type: NodeType,
}

impl From<NodeType> for NodeMetaData {
    fn from(node_type: NodeType) -> Self {
        NodeMetaData { node_type }
    }
}

pub(crate) struct ExecutionNode {
    pub(crate) metadata: NodeMetaData,
    pub(crate) compiled_rule: RuleFragment,
    pub(crate) rule: String,
    pub(crate) children: Vec<ExecutionNode>,
}

fn constraint_node(constraint: &Constraint) -> ExecutionNode {
    let rule = upgrade_constraint(constraint);
    let compiled_rule = compile_rule(rule.as_str()).expect("Failed to compile rule");
    ExecutionNode {
        rule,
        compiled_rule,
        children: vec![],
        metadata: NodeMetaData::from(NodeType::Constraint),
    }
}

fn strategy_node(strategy: &Strategy, segment_map: &HashMap<i32, Segment>) -> ExecutionNode {
    let rule = upgrade_strategy(strategy, segment_map);
    let compiled_rule = compile_rule(rule.as_str()).expect("Failed to compile rule");

    let children = strategy
        .constraints
        .clone()
        .unwrap_or_default()
        .iter()
        .map(constraint_node)
        .collect();

    ExecutionNode {
        rule,
        compiled_rule,
        children,
        metadata: NodeMetaData::from(NodeType::Strategy),
    }
}

fn toggle_node(feature: &ClientFeature, segment_map: &HashMap<i32, Segment>) -> ExecutionNode {
    let rule = if feature.enabled {
        "true".into()
    } else {
        "false".into()
    };

    let enabled = feature.enabled;
    let compiled_rule: RuleFragment = Box::new(move |_| enabled);

    let children: Vec<ExecutionNode> = feature
        .strategies
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|strategy| strategy_node(strategy, segment_map))
        .collect();

    ExecutionNode {
        rule,
        compiled_rule,
        children,
        metadata: NodeMetaData::from(NodeType::Toggle),
    }
}

pub(crate) fn build_execution_tree(
    feature: &ClientFeature,
    segment_map: &HashMap<i32, Segment>,
) -> ExecutionNode {
    toggle_node(feature, segment_map)
}
