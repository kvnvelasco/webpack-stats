static SOURCE_FILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test_projects/v5/compilation-stats.json"
));

#[test]
fn full_deserialization() {
    let value: super::Stats = serde_json::from_str(SOURCE_FILE).expect("Does serde");
}
