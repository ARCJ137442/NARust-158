//! NARS中有关「推理」的内容
//! * 🚩【2024-05-02 15:54:15】计划通过「全有默认实现的模板特征」作为功能实现方法
//!
//! # 📄OpenNARS
//!
//! The entry point of the package is `RuleTables`, which dispatch the premises (a task, and maybe also a belief) to various rules, according to their type combination.
//!
//! There are four major groups of inference rules:
//!
//! 1. `LocalRules`, where the task and belief contains the same pair of terms, and the rules provide direct solutions to problems, revise beliefs, and derive some conclusions;
//! 2. `SyllogisticRules`, where the task and belief share one common term, and the rules derive conclusions between the other two terms;
//! 3. `CompositionalRules`, where the rules derive conclusions by compose or decompose the terms in premises, so as to form new terms that are not in the two premises;
//! 4. `StructuralRules`, where the task derives conclusions all by itself, while the other "premise" serves by indicating a certain syntactic structure in a compound term.
//!
//! In the system, forward inference (the task is a Judgement) and backward inference (the task is a Question) are mostly isomorphic to each other, so that the inference rules produce conclusions with the same content for different types of tasks. However, there are exceptions. For example, backward inference does not generate compound terms.
//!
//! There are three files containing numerical functions:
//!
//! 1. `TruthFunctions`: the functions that calculate the truth value of the derived judgements and the desire value (a variant of truth value) of the derived goals;
//! 2. `BudgetFunctions`: the functions that calculate the budget value of the derived tasks, as well as adjust the budget value of the involved items (concept, task, and links);
//! 3. `UtilityFunctions`: the common basic functions used by the others.
//!
//! In each case, there may be multiple applicable rules, which will be applied in parallel. For each rule, each conclusion is formed in three stages, to determine (1) the content (as a Term), (2) the truth-value, and (3) the budget-value, roughly in that order.

nar_dev_utils::mods! {
    // 🆕推理上下文 [`ReasonContext`](reason_context::ReasonContext)
    pub use reason_context;

    // 🆕推导上下文 [`DerivationContext`](derivation_context::DerivationContext)
    pub use derivation_context;

    // 规则表 `RuleTables`
    pub use rule_tables;

    // 本地规则 `LocalRules`
    pub use local_rules;

    // 三段论规则 `SyllogisticRules`
    pub use syllogistic_rules;

    // 组合规则 `CompositionalRules`
    pub use compositional_rules;

    // 结构规则 `StructuralRules`
    pub use structural_rules;

    // 实用函数 `UtilityFunctions`
    pub use utility_functions;

    // 预算值函数 `BudgetFunctions`
    pub use budget_functions;

    // 真值函数 `TruthFunctions`
    pub use truth_functions;
}
