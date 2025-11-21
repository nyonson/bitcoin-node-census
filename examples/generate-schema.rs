use bitcoin_node_census::report::CensusReport;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(CensusReport);
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
    );
}
