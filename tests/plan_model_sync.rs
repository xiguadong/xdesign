use std::{collections::BTreeSet, fs, path::Path};
use xdesign::simple_flow::designed_simple_flow_plan;
use xdesign::wish_verify::wish_verify_flow_plan;

#[test]
fn frozen_plan_rule_conditions_and_reads_match_the_sysml_design() {
    let model = fs::read_to_string("mbse/model/detailedDesign.sysml").unwrap();
    let plan = designed_simple_flow_plan();
    for checker in plan
        .checkpoints
        .iter()
        .filter_map(|gate| gate.checker.as_ref())
    {
        for rule in &checker.rules {
            let marker = format!("attribute redefines ruleId = \"{}\";", rule.rule_id);
            let mut matches = model.match_indices(&marker);
            let (offset, _) = matches
                .next()
                .unwrap_or_else(|| panic!("missing model rule {}", rule.rule_id));
            assert!(
                matches.next().is_none(),
                "duplicate model rule {}",
                rule.rule_id
            );
            let following = model[offset + marker.len()..]
                .split_once("\n            }")
                .map(|(block, _)| block)
                .unwrap_or_else(|| panic!("unclosed model rule {}", rule.rule_id));
            let condition_start = "attribute redefines condition = \"";
            let condition = following
                .split_once(condition_start)
                .and_then(|(_, rest)| rest.split_once("\";"))
                .map(|(value, _)| value)
                .unwrap_or_else(|| panic!("missing model condition for {}", rule.rule_id));
            assert_eq!(
                rule.condition, condition,
                "frozen plan condition differs from model rule {}",
                rule.rule_id
            );
            let reads = following
                .split_once("attribute redefines reads = ")
                .and_then(|(_, rest)| rest.split_once(';'))
                .map(|(value, _)| {
                    value
                        .split('"')
                        .enumerate()
                        .filter_map(|(index, part)| (index % 2 == 1).then_some(part.to_owned()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| panic!("missing model reads for {}", rule.rule_id));
            assert_eq!(
                rule.reads, reads,
                "frozen plan reads differ from model rule {}",
                rule.rule_id
            );
        }
    }
}

#[test]
fn simple_flow_file_plan_covers_the_source_and_test_inventory() {
    let model = fs::read_to_string("mbse/model/detailedDesign.sysml").unwrap();
    let file_plan = model
        .split_once("item simpleFlowFilePlan {")
        .map(|(_, rest)| rest.split_once("view simpleFlowFilePlanView").unwrap().0)
        .unwrap();
    let marker = "attribute redefines repositoryPath = \"";
    let declared: BTreeSet<_> = file_plan
        .match_indices(marker)
        .map(|(offset, _)| {
            file_plan[offset + marker.len()..]
                .split_once('"')
                .unwrap()
                .0
                .to_owned()
        })
        .collect();
    let mut actual = BTreeSet::from(["Cargo.toml".to_owned()]);
    for directory in ["src", "tests", "tools/checks"] {
        let mut directories = vec![Path::new(directory).to_path_buf()];
        while let Some(current) = directories.pop() {
            for entry in fs::read_dir(current).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    directories.push(path);
                } else if matches!(
                    path.extension().and_then(|extension| extension.to_str()),
                    Some("rs" | "py")
                ) {
                    let owned = path.to_string_lossy().into_owned();
                    // The wish-verify slice keeps its own file plan and sync test.
                    if owned != "tests/wish_verify.rs" && !owned.starts_with("src/wish_verify/") {
                        actual.insert(owned);
                    }
                }
            }
        }
    }
    assert_eq!(
        declared, actual,
        "model file plan differs from source/test inventory"
    );
}

#[test]
fn wish_verify_file_plan_covers_the_flow_module_inventory() {
    let model = fs::read_to_string("mbse/model/detailedDesign.sysml").unwrap();
    let file_plan = model
        .split_once("item wishVerifyFlowFilePlan {")
        .map(|(_, rest)| {
            rest.split_once("view wishVerifyFlowFilePlanView")
                .unwrap()
                .0
        })
        .unwrap();
    let marker = "attribute redefines repositoryPath = \"";
    let declared: BTreeSet<_> = file_plan
        .match_indices(marker)
        .map(|(offset, _)| {
            file_plan[offset + marker.len()..]
                .split_once('"')
                .unwrap()
                .0
                .to_owned()
        })
        .collect();
    for path in &declared {
        assert!(
            Path::new(path).is_file(),
            "planned file {path} does not exist"
        );
    }
    let mut actual = BTreeSet::new();
    let mut directories = vec![Path::new("src/wish_verify").to_path_buf()];
    while let Some(current) = directories.pop() {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                actual.insert(path.to_string_lossy().into_owned());
            }
        }
    }
    actual.insert("tests/wish_verify.rs".to_owned());
    assert_eq!(
        declared, actual,
        "model wish-verify file plan differs from the flow module inventory"
    );
}

#[test]
fn wish_verify_gate_criteria_match_the_sysml_design() {
    fn gate_criteria(model: &str, gate: &str) -> Vec<String> {
        let marker = format!("part {gate} : ");
        let mut matches = model.match_indices(&marker);
        let (offset, _) = matches
            .next()
            .unwrap_or_else(|| panic!("missing model gate {gate}"));
        assert!(matches.next().is_none(), "duplicate model gate {gate}");
        model[offset..]
            .split_once("attribute redefines criteria = ")
            .and_then(|(_, rest)| rest.split_once(");"))
            .map(|(value, _)| {
                value
                    .split('"')
                    .enumerate()
                    .filter_map(|(index, part)| (index % 2 == 1).then_some(part.to_owned()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| panic!("missing model criteria for {gate}"))
    }
    let design = fs::read_to_string("mbse/model/detailedDesign.sysml").unwrap();
    let architecture = fs::read_to_string("mbse/model/architecture.sysml").unwrap();
    let plan = wish_verify_flow_plan();
    assert_eq!(plan.checkpoints[0].checkpoint_id, "wishAccepted");
    assert_eq!(plan.checkpoints[1].checkpoint_id, "outcomeAccepted");
    assert_eq!(
        plan.checkpoints[0].criteria,
        gate_criteria(&design, "wishAccepted"),
        "blueprint criteria differ from the designed wishAccepted gate"
    );
    assert_eq!(
        plan.checkpoints[1].criteria,
        gate_criteria(&architecture, "outcomeAccepted"),
        "blueprint criteria differ from the architectural outcomeAccepted gate"
    );
}
