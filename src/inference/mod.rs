//! OpenNARS `nars.inference`
//! * 🚩【2024-05-02 15:54:15】计划通过「全有默认实现的模板特征」作为功能实现方法
//!

nar_dev_utils::mods! {
    // 🆕「证据数值」
    pub use evidence_real;

    // 实用函数 `UtilityFunctions`
    pub use utility_functions;

    // 预算值函数 `BudgetFunctions`
    pub use budget_functions;

    // `TruthFunctions`
    pub use truth_functions;

    // 规则表 `RuleTables`
    pub use rule_tables;

    // 三段论规则 `SyllogisticRules`
    pub use syllogistic_rules;

    // 本地规则 `LocalRules`
    pub use local_rules;

    // 组合规则 `CompositionalRules`
    pub use compositional_rules;

    // 结构规则 `StructuralRules`
    pub use structural_rules;
}
