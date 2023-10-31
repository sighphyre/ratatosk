extern crate clap;

use std::io::{self, Read};
use unleash_types::client_features::ClientFeatures;
use unleash_yggdrasil::{Context, EngineState};

fn main() {
    let mut input = String::new();

    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");
    let toggles =
        serde_json::from_str::<ClientFeatures>(&input).expect("Failed to parse input as JSON");

    let mut engine = EngineState::default();
    engine.take_state(toggles);

    println!(
        "Wheee {:?}",
        engine.is_enabled("Feature.A", &Context::default())
    );
}
