//! NARS中有关「推理」的内容
//! * 🚩【2024-05-02 15:54:15】计划通过「全有默认实现的模板特征」作为功能实现方法
//! * ♻️【2024-05-16 14:01:02】将混杂的推理控制过程分类放置
//!   * 🚩与「上下文」有关的放在一块：推理上下文、推理上下文……
//!   * 🚩与「概念」「记忆区」有关的放在一块：概念处理、记忆区处理……
//!   * 🚩与「推理规则」有关的放在一块：本地规则、三段论规则……
//!   * 🚩与「推理函数」有关的放在一块：真值函数、预算函数……
//! * 🚩【2024-05-22 01:35:53】现在将与「推理周期」有关的「推理控制机制」移至[`crate::control`]中
//!   * 📌目前将只留下纯粹的「推理规则」与「推导函数」
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
    // 🆕特征：真值、预算、证据基……
    pub use traits;

    // ♻️数值函数：真值函数、预算函数……
    pub use functions;

    // 🛠️预算推理：预算×推理上下文
    pub use budget_inference;

    // 📥本地推理：增删信念、答问……
    pub use local_inference;

    // 🏗️推理引擎：可配置推理功能表
    pub use engine;

    // ♻️具体规则：直接推理、转换推理、匹配推理、概念推理……
    pub use rules;
}

// 测试工具与测试集
// * 📌【2024-08-24 11:41:01】目前仅在库的内部使用
//   * ℹ️对外接口可能不稳定
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::*;
