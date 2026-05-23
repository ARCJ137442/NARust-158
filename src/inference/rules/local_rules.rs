//! 🎯复刻OpenNARS `nars.inference.LocalRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::type_context)
//!
//! ## Logs
//!
//! * ✅【2024-05-07 18:51:30】初步实现方法API（函数签名、文档、源码附注）
//! * ♻️【2024-06-30 11:00:41】开始根据改版OpenNARS重写

use crate::{
    control::{util_outputs, ContextDerivation, ReasonContext},
    entity::{BudgetValue, Judgement, JudgementV1, Sentence, ShortFloat, Task},
    global::Float,
    inference::{Budget, BudgetFunctions},
};
use navm::output::Output;

/// [`try_solution`]的复合返回值
/// * 📌标明其中的所有参数语义
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub(in crate::inference) enum SolutionResult {
    /// 驳回
    /// * 📄新解差于旧解
    Rejected,
    /// 新解
    /// * 📌需要将传入的`belief`确立为「新的最优解」
    NewSolution {
        /// 更新后的「问题任务」优先级
        updated_question_priority: ShortFloat,
        /// 「激活任务」所需的预算值与「候选信念」
        /// * `.is_some()` = 是否需要调用者（上下文）「激活任务」
        params_to_activate_task: Option<(BudgetValue, JudgementV1)>,
        /// 可能导出的「回答/完成」输出
        new_output: Option<Output>,
    },
}

/// 尝试对「问题任务」求解
/// * 🚩【2024-06-30 11:31:00】此处不再引入「推理上下文」，以便在「问题任务」中解耦
#[must_use]
pub(in crate::inference) fn try_solution_calculate(
    belief: &impl Judgement,
    question_task: &Task,
    budget_threshold: Float, // 在「激活任务」时使用
) -> SolutionResult {
    use SolutionResult::*;
    // * 🚩预设&断言
    debug_assert!(question_task.is_question(), "要解决的必须是「问题」");
    let problem = question_task.as_question().unwrap();

    // * 🚩验证这个信念是否为「解决问题的最优解」
    let new_q = BudgetValue::solution_quality(question_task, belief);

    if let Some(old_best) = question_task.best_solution() {
        let old_q = BudgetValue::solution_quality(question_task, old_best);
        // * 🚩新解比旧解还差⇒驳回
        if old_q >= new_q {
            return Rejected;
        }
    }

    // * 🚩若比先前「最优解」还优，那就确立新的「最优解」
    let new_output = match question_task.is_input() {
        // moved from Sentence
        // * 🚩同时在此确立「回答」：只在回应「输入的任务」时反映
        true => Some(util_outputs::output_answer(belief)),
        false => None,
    };

    // * 🚩计算新预算值
    let budget = BudgetValue::solution_eval(problem, belief, question_task);
    // * 🚩计算「候选信念」
    // * 📝在「解决问题」时，需要使用「当前问题的上游信念」作推断
    let parent_belief = question_task.parent_belief();
    // * 🚩预备「问题任务」的预算值（优先级）
    // * 📝解决问题后，在「已解决的问题」之预算中 降低（已经解决了，就将算力多留到「未解决问题」上）
    // * 📌【2024-06-30 11:25:23】断言：此处的`newQ`就是`solutionQuality`
    let updated_question_priority = ShortFloat::min(question_task.priority(), !new_q);
    // * 🚩计算「是否要激活任务」并返回其中的预算值
    let params_to_activate_task = match budget.budget_above_threshold(budget_threshold) {
        true => parent_belief.map(|belief| (budget, belief.clone())),
        false => None,
    };

    // * 🚩最后返回枚举变种「新解」
    NewSolution {
        new_output,
        updated_question_priority,
        params_to_activate_task,
    }
}

/// 将上述结果应用到「当前任务」中
/// * 🚩要求输入选定的「最优解」以利用引用（难以将引用放到结构体中）
/// * 🚩【2024-06-30 11:48:08】只能存在一个函数指针：调用方不能重复借用，且不知此处是先后调用
/// * 🚩方法应用顺序：先`task`后`context`
///   * 1 更新「问题任务」的解
///   * 2 更新「推理上下文」——激活任务
pub(in crate::inference) fn try_solution_apply_task(
    result: &SolutionResult,
    question_task: &mut Task,
    solution: &JudgementV1,
) {
    use SolutionResult::*;
    match result {
        // * 🚩驳回⇒直接返回
        Rejected => {}
        // * 🚩新解⇒应用新解
        NewSolution {
            updated_question_priority,
            ..
        } => {
            // * 🚩设置最优解
            question_task.set_best_solution(solution.clone());
            // * 🚩设置新优先级
            question_task.set_priority(*updated_question_priority);
        }
    }
}

/// 将上述结果应用到推理上下文中
/// * 📌通过函数指针实现「借用分离」
/// * 🚩要求输入选定的「最优解」以利用引用（难以将引用放到结构体中）
/// * 🚩【2024-06-30 11:48:08】只能存在一个函数指针：调用方不能重复借用，且不知此处是先后调用
pub(in crate::inference) fn try_solution_apply_context(
    result: SolutionResult,
    solution: &JudgementV1,
    context: &mut impl ReasonContext,
) {
    use SolutionResult::*;
    match result {
        // * 🚩驳回⇒直接返回
        Rejected => {}
        // * 🚩新解⇒应用新解
        NewSolution {
            new_output,
            params_to_activate_task,
            ..
        } => {
            // * 🚩尝试「激活任务」
            if let Some((budget, candidate_belief)) = params_to_activate_task {
                context.activated_task(budget, solution, &candidate_belief);
            }
            // * 🚩报告输出
            if let Some(output) = new_output {
                context.report(output);
            }
        }
    }
}
