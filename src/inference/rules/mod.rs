//! NARS中具体的「推理规则」
//!
//! ## Logs
//!
//! * 🚩【2024-05-16 14:04:59】重构并独立成单独的子模块
//! * ♻️【2024-06-26 12:08:43】开始根据改版OpenNARS重写

nar_dev_utils::mods! {
    // 实用工具
    use utils;

    // 规则表 `RuleTables`
    pub use table;

    // 本地规则 `LocalRules`
    pub use local_rules;

    // 三段论规则 `SyllogisticRules`
    use syllogistic_rules;

    // 组合规则 `CompositionalRules`
    use compositional_rules;

    // 结构规则 `StructuralRules`
    use structural_rules;

    // 🆕匹配规则 `MatchingRules`
    pub use matching_rules;

    // 🆕转换规则 `TransformRules`
    pub use transform_rules;
}
