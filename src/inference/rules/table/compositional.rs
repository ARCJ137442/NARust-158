//! 组合规则的「子分派函数」
//! * 🎯包括「不直接涉及推理结论」的诸多方法

use super::syllogistic::*;
use crate::{
    control::*,
    entity::*,
    inference::rules::{utils::*, *},
    io::symbols::{IMPLICATION_RELATION, INHERITANCE_RELATION, SIMILARITY_RELATION},
    language::{variable_process, CompoundTerm, Statement, Term},
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;


/// # 📄OpenNARS
///
/// ```nal
/// {<S ==> M>, <P ==> M>} |- {
/// <(S|P) ==> M>, <(S&P) ==> M>,
/// <(S-P) ==> M>, <(P-S) ==> M>
/// }
/// ```
pub fn compose_compound(
    task_content: Statement,
    belief_content: Statement,
    shared_term_i: usize,
    context: &mut ReasonContextConcept,
) {
    // * 🚩前提：任务是判断句（前向推理）、任务与信念类型相同
    // * 🚩提取词项
    // * 🚩预判，分派到「解构」中
    // * 🚩「任务词项中的另一项」包含「信念词项的另一侧」的所有元素
    // * 🚩「信念词项中的另一项」包含「任务词项的另一侧」的所有元素
    // * 🚩NAL-3规则：交并差
    // * 🚩引入变量
    // introVarImage(taskContent, beliefContent, index);
    // TODO
}
