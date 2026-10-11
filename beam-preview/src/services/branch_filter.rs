// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/services/branch_filter.rs
// 📌 Amac: GitHub'dan gelen salt-okunur dallari proje politikasina gore filtreler
// 📌 Modul - Service Rust
// Version: 0.1.0
// Aciklama: Normal gorunumde etkin dallar, gelismis gorunumde tum gercek dallar listelenir
// Bagimli Oldugu Katman: Config | Tool | View

use crate::config::branch_visibility;
use crate::BranchSummary;

pub fn apply(
    project: &str,
    include_all: bool,
    branches: Vec<BranchSummary>,
) -> Vec<BranchSummary> {
    if include_all {
        return branches;
    }
    let allowed = branch_visibility::default_branches(project);
    branches
        .into_iter()
        .filter(|branch| allowed.contains(&branch.name.as_str()))
        .collect()
}
