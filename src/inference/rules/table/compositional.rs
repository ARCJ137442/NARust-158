//! 组合规则的「子分派函数」
//! * 🎯包括「不直接涉及推理结论」的诸多方法

use crate::{
    control::*,
    entity::Sentence,
    inference::rules::{utils::*, *},
    language::*,
};
use nar_dev_utils::RefCount;

/// 🆕原OpenNARS规则，现成为一个纯分派函数
/// * ℹ️所直接包含的规则，请移步至[`crate::inference::rules::compositional_rules::compose_as_set`]
pub fn compose_compound(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // * 🚩前提：任务是判断句、任务与信念类型相同
    // * 📝【2024-08-07 17:22:44】经OpenNARS 3.0.4验证：必须只能是判断句
    if !context.current_task().get_().is_judgement() || !task_content.is_same_type(&belief_content)
    {
        return;
    }

    // * 🚩提取词项
    let [component_common, component_t] = shared_term_i.select(task_content.sub_pre());
    let component_b = shared_term_i
        .opposite()
        .select_one(belief_content.sub_pre());
    // * 🚩预判，分派到「解构」中
    match [component_t.as_compound(), component_b.as_compound()] {
        // * 🚩「任务词项中的另一项」包含「信念词项的另一侧」的所有元素
        [Some(component_t), _] if component_t.contain_all_components(component_b) => {
            return decompose_as_set(
                task_content,
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
            return decompose_as_set(
                task_content,
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
        belief_content,
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
