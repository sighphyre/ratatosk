use std::collections::HashMap;

use unleash_types::client_features::{ClientFeature, Constraint, Operator, Segment, Strategy};
use unleash_yggdrasil::{
    strategy_parsing::{compile_rule, RuleFragment},
    strategy_upgrade::{upgrade_constraint, upgrade_strategy},
};

trait PrettyPrintable {
    fn pretty_print(&self) -> String;
}

impl PrettyPrintable for ClientFeature {
    fn pretty_print(&self) -> String {
        self.name.clone()
    }
}

impl PrettyPrintable for Strategy {
    fn pretty_print(&self) -> String {
        camel_case_to_sentence(&self.name)
    }
}

impl PrettyPrintable for Constraint {
    fn pretty_print(&self) -> String {
        match &self.operator {
            Operator::NotIn => "Not in",
            Operator::In => "In",
            Operator::StrEndsWith => "Any string that ends with",
            Operator::StrStartsWith => "Any string that starts with",
            Operator::StrContains => "Any string that contains",
            Operator::NumEq => "Number equals",
            Operator::NumGt => "Number greater than",
            Operator::NumGte => "Number greater than or equal to",
            Operator::NumLt => "Number less than",
            Operator::NumLte => "Number less than or equal to",
            Operator::DateAfter => "Date after",
            Operator::DateBefore => "Date before",
            Operator::SemverEq => "Semver equals",
            Operator::SemverLt => "Semver less than",
            Operator::SemverGt => "Semver greater than",
            Operator::Unknown(name) => name,
        }
        .into()
    }
}

// I stole this from ChatGPT, it needs validation
fn camel_case_to_sentence(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i == 0 {
            result.push(c.to_uppercase().next().unwrap()); // Capitalize the first letter
        } else if c.is_uppercase() {
            result.push(' '); // Add a space before uppercase letters (but not the first one)
            result.push(c.to_uppercase().next().unwrap());
        } else {
            result.push(*c);
        }
    }

    result
}

#[derive(Debug, Clone)]
pub(crate) enum NodeType {
    Toggle,
    Strategy,
    Constraint,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeMetaData {
    pub(crate) node_type: NodeType,
    pub(crate) friendly_name: String,
}

impl NodeMetaData {
    fn from(node_type: NodeType, name: &str) -> Self {
        NodeMetaData {
            node_type,
            friendly_name: name.into(),
        }
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
        metadata: NodeMetaData::from(NodeType::Constraint, constraint.pretty_print().as_str()),
    }
}

fn strategy_node(strategy: &Strategy, segment_map: &HashMap<i32, Segment>) -> ExecutionNode {
    let strategy_without_constraints = Strategy {
        constraints: None,
        ..strategy.clone()
    };

    let rule = upgrade_strategy(&strategy_without_constraints, segment_map);
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
        metadata: NodeMetaData::from(NodeType::Strategy, strategy.pretty_print().as_str()),
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
        metadata: NodeMetaData::from(NodeType::Toggle, feature.pretty_print().as_str()),
    }
}

pub(crate) fn build_execution_tree(
    feature: &ClientFeature,
    segment_map: &HashMap<i32, Segment>,
) -> ExecutionNode {
    toggle_node(feature, segment_map)
}
