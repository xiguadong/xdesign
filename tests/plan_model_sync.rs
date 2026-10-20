use simpflow::designed_simple_flow_plan;
use std::fs;

#[test]
fn frozen_plan_rule_conditions_match_the_sysml_design() {
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
            let following = &model[offset + marker.len()..];
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
        }
    }
}
