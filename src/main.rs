extern crate clap;

use std::{
    collections::HashMap,
    io::{self, Read},
};
use unleash_types::client_features::{
    ClientFeature, ClientFeatures, Constraint, Segment, Strategy,
};
use unleash_yggdrasil::{
    state::EnrichedContext,
    strategy_upgrade::{upgrade_constraint, upgrade_strategy},
    Context,
};

#[derive(Debug)]
enum FragmentType {
    Toggle,
    Strategy,
    Constraint,
}

#[derive(Debug)]
struct TraversalResult {
    fragment_type: FragmentType,
    rule: String,
}

enum ChainType {
    None,
    Or,
    And,
}

struct ExecutionNode {
    rule: String,
    children: Vec<ExecutionNode>,
    chaining: ChainType,
}

fn constraint_node(constraint: &Constraint) -> ExecutionNode {
    let _rule = upgrade_constraint(constraint);
    ExecutionNode {
        rule: "constraint".into(),
        children: vec![],
        chaining: ChainType::None,
    }
}

fn strategy_node(strategy: &Strategy, segment_map: &HashMap<i32, Segment>) -> ExecutionNode {
    let rule = upgrade_strategy(strategy, segment_map);

    let children = strategy
        .constraints
        .clone()
        .unwrap_or_default()
        .iter()
        .map(constraint_node)
        .collect();

    ExecutionNode {
        rule,
        children,
        chaining: ChainType::And,
    }
}

fn toggle_node(feature: &ClientFeature, segment_map: &HashMap<i32, Segment>) -> ExecutionNode {
    let rule = if feature.enabled {
        "true".into()
    } else {
        "false".into()
    };

    let children: Vec<ExecutionNode> = feature
        .strategies
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|strategy| strategy_node(strategy, segment_map))
        .collect();

    ExecutionNode {
        rule,
        children,
        chaining: ChainType::Or,
    }
}

fn build_execution_tree(
    feature: &ClientFeature,
    segment_map: &HashMap<i32, Segment>,
) -> Vec<TraversalResult> {
    let _node = toggle_node(feature, segment_map);

    let mut traversal_items: Vec<TraversalResult> = vec![];

    if feature.enabled {
        traversal_items.push(TraversalResult {
            rule: "true".into(),
            fragment_type: FragmentType::Toggle,
        });
    } else {
        traversal_items.push(TraversalResult {
            rule: "false".into(),
            fragment_type: FragmentType::Toggle,
        });
    }

    let base_context = Context::default();
    let _enriched_context = EnrichedContext::from(base_context, feature.name.clone());

    for strategy in feature.strategies.clone().unwrap_or_default() {
        let strategy_without_constraints = Strategy {
            constraints: None,
            ..strategy.clone()
        };

        traversal_items.push(TraversalResult {
            rule: upgrade_strategy(&strategy_without_constraints, &segment_map),
            fragment_type: FragmentType::Strategy,
        });

        for _constraint in strategy.constraints.unwrap_or_default() {}
    }

    traversal_items

    // traversal_items.append(other)
}

// fn build_execution_tree(feature: Option<&ClientFeature>, segment_map: &HashMap<i32, Segment>) {}

fn main() {
    let mut input = String::new();

    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");
    let features =
        serde_json::from_str::<ClientFeatures>(&input).expect("Failed to parse input as JSON");

    let feature = features
        .features
        .iter()
        .find(|f| f.name == "Feature.A")
        .expect("Failed to find feature by name")
        .clone();

    let segment_map: HashMap<i32, Segment> = features
        .segments
        .unwrap_or_default()
        .iter()
        .map(|segment| (segment.id, segment.clone()))
        .collect();

    let explanation = build_execution_tree(&feature, &segment_map);

    println!("Explanation: {:#?}", explanation);

    let _t = ExecutionNode {
        rule: "true".into(),
        children: vec![ExecutionNode {
            rule: "false".into(),
            children: vec![],
            chaining: ChainType::None,
        }],
        chaining: ChainType::None,
    };
}
