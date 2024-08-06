//! 组合规则的「子分派函数」
//! * 🎯包括「不直接涉及推理结论」的诸多方法

use crate::{
    control::*,
    inference::rules::{compose_as_set, decompose_compound, intro_var_outer, utils::*},
    language::*,
};
use ReasonDirection::*;

/// # 📄OpenNARS
///
/// ```nal
/// {<S ==> M>, <P ==> M>} |- {
/// <(S|P) ==> M>, <(S&P) ==> M>,
/// <(S-P) ==> M>, <(P-S) ==> M>
/// }
/// ```
pub fn compose_compound(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩前提：任务是判断句（前向推理）、任务与信念类型相同
    if context.reason_direction() != Forward || !task_content.is_same_type(&belief_content) {
        return;
    }

    // * 🚩提取词项
    let [component_common, component_t] = shared_term_i.select_and_other(task_content.sub_pre());
    let component_b = shared_term_i.opposite().select(belief_content.sub_pre());
    // * 🚩预判，分派到「解构」中
    match [component_t.as_compound(), component_b.as_compound()] {
        // * 🚩「任务词项中的另一项」包含「信念词项的另一侧」的所有元素
        [Some(component_t), _] if component_t.contain_all_components(component_b) => {
            return decompose_compound(
                component_t,
                component_b,
                component_common,
                shared_term_i,
                PremiseSource::Task,
                context,
            )
        }
        // * 🚩「信念词项中的另一项」包含「任务词项的另一侧」的所有元素
        [_, Some(component_b)] if component_b.contain_all_components(component_t) => {
            return decompose_compound(
                component_b,
                component_t,
                component_common,
                shared_term_i,
                PremiseSource::Belief,
                context,
            )
        }
        _ => {}
    }
    // * 🚩NAL-3规则：交并差
    compose_as_set(
        task_content,
        shared_term_i,
        component_common,
        component_t,
        component_b,
        context,
    );
    // * 🚩引入变量
    if task_content.instanceof_inheritance() {
        intro_var_outer(task_content, belief_content, shared_term_i, context);
        // intro_var_image(task_content, belief_content, shared_term_i, context);
    }
}
