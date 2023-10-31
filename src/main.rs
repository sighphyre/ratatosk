extern crate clap;

mod compile;
mod explain;

use compile::build_execution_tree;
use std::{
    collections::HashMap,
    io::{self, Read},
};
use unleash_types::client_features::{
    ClientFeature, ClientFeatures, Constraint, Segment, Strategy,
};
use unleash_yggdrasil::{
    state::EnrichedContext,
    strategy_parsing::{compile_rule, RuleFragment},
    strategy_upgrade::{upgrade_constraint, upgrade_strategy},
};

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
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use unleash_types::client_features::{ClientFeature, ClientFeatures, Segment};
    use unleash_yggdrasil::{state::EnrichedContext, Context};

    use crate::{build_execution_tree, explain::Executable};

    fn destructure_feature(
        feature_name: &str,
        raw_features: &str,
    ) -> (ClientFeature, HashMap<i32, Segment>) {
        let features = serde_json::from_str::<ClientFeatures>(raw_features)
            .expect("Failed to parse input as JSON");

        let feature = features
            .features
            .iter()
            .find(|f| f.name == feature_name)
            .expect("Failed to find feature by name")
            .clone();

        let segment_map: HashMap<i32, Segment> = features
            .segments
            .unwrap_or_default()
            .iter()
            .map(|segment| (segment.id, segment.clone()))
            .collect();

        (feature, segment_map)
    }

    #[test]
    fn does_the_thing() {
        let test_data = include_str!("../testdata/simple.json");
        let (feature, segments) = destructure_feature("Feature.A", test_data);

        let context = EnrichedContext::from(Context::default(), "Feature.A".into());

        let tree = build_execution_tree(&feature, &segments);

        let results = tree.execute(&context);
        println!("{:#?}", results);

        panic!("")
    }
}
