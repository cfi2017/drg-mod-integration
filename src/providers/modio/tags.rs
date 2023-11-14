use std::collections::{BTreeSet, HashSet};
use crate::providers::{ApprovalStatus, ModioTags, RequiredStatus};

pub fn process_modio_tags(set: &HashSet<String>) -> ModioTags {
    let qol = set.contains("QoL");
    let gameplay = set.contains("Gameplay");
    let audio = set.contains("Audio");
    let visual = set.contains("Visual");
    let framework = set.contains("Framework");
    let required_status = if set.contains("RequiredByAll") {
        RequiredStatus::RequiredByAll
    } else {
        RequiredStatus::Optional
    };
    let approval_status = if set.contains("Verified") || set.contains("Auto-Verified") {
        ApprovalStatus::Verified
    } else if set.contains("Approved") {
        ApprovalStatus::Approved
    } else {
        ApprovalStatus::Sandbox
    };
    // Basic heuristic to collect all the tags which begin with a number, like `1.38`.
    let versions = set
        .iter()
        .filter(|i| i.starts_with(char::is_numeric))
        .cloned()
        .collect::<BTreeSet<String>>();

    ModioTags {
        qol,
        gameplay,
        audio,
        visual,
        framework,
        versions,
        required_status,
        approval_status,
    }
}
