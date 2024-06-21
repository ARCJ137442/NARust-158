//! 可被存放的「实体」
//!
//! # 📄OpenNARS
//!
//! Data entities that are independently stored
//!
//! * `ShortFloat`: `BudgetValue` (priority/durability/quality) and `TruthValue` (frequency/confidence)
//! * `Stamp`: serial numbers and creation time associated to a TruthValue
//! * `Sentence`: a Term, a TruthValue, and a Stamp. A Sentence can be a Judgement, a Goal, or a Question.
//! * `Task`: a Sentence to be processed.
//! * `TermLink`: built in pair between a compound term and a component term.
//! * `TaskLink`: special TermLink referring to a Task, whose Term equals or contains the current Term.
//! * `Concept`: labeled by a Term, contains a TaskLink bag and a TermLink bag for indirect tasks/beliefs, as well as beliefs/questions/goals directly on the Term.
//! * `Item`: Concept, Task, or TermLink
//!
//! in NARS, each task is processed in two stages:
//!
//! 1. Direct processing by matching, in the concept corresponding to the content, in one step. It happens when the task is inserted into memory.
//! 2. Indirect processing by reasoning, in related concepts and unlimited steps. It happens in each inference cycle.

// 有关「浮点数组」的结构
mod float_values;
pub use float_values::*;

// 时间戳 `Stamp` | 内有导出宏定义
mod stamp;
pub use stamp::*;

// 语句 `Sentence`
mod sentence;
pub use sentence::*;

// 物品 `Item`
mod item;
pub use item::*;

// 任务 `Task`
mod task;
pub use task::*;

// 链接 | 词项链、任务链 等
mod linkages;
pub use linkages::*;

// nar_dev_utils::mods! {
//     // 词项链 `TermLink`
//     pub use term_link;

//     // 任务链 `TaskLink`
//     pub use task_link;

//     // 概念 `Concept`
//     pub use concept;
// }
