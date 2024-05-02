//! 可被存放的「实体」
//!
//! # 📄OpenNARS
//!
//! Data entities that are independently stored
//!
//! * ShortFloats: BudgetValue (priority/durability/quality) and TruthValue (frequency/confidence)
//! * Stamp: serial numbers and creation time associated to a TruthValue
//! * Sentence: a Term, a TruthValue, and a Stamp. A Sentence can be a Judgment, a Goal, or a Question.
//! * Task: a Sentence to be processed.
//! * TermLink: built in pair between a compound term and a component term.
//! * TaskLink: special TermLink referring to a Task, whose Term equals or contains the current Term.
//! * Concept: labeled by a Term, contains a TaskLink bag and a TermLink bag for indirect tasks/beliefs, as well as beliefs/questions/goals directly on the Term.
//! * Item: Concept, Task, or TermLink
//!
//! in NARS, each task is processed in two stages:
//!
//! 1. Direct processing by matching, in the concept corresponding to the content, in one step. It happens when the task is inserted into memory.
//! 2. Indirect processing by reasoning, in related concepts and unlimited steps. It happens in each inference cycle.

nar_dev_utils::mods! {
    // 短浮点
    pub use short_float;
    // 预算值
    pub use budget;
    // 物品
    pub use item;
}
