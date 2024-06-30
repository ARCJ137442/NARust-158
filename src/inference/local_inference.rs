//! 本地推理
//! * 🎯承载原先「直接推理」的部分
//! * 📝其中包含「修订规则」等

use crate::{
    control::{util_outputs, ReasonContext, ReasonContextDirect},
    entity::{BudgetValue, Concept, Judgement, Punctuation, Sentence, ShortFloat, Task},
    inference::{try_solution_apply, try_solution_calculate, Budget, BudgetFunctions, Evidential},
    language::Term,
    util::{Iterable, RefCount, ToDisplayAndBrief},
};

/// 本地推理 入口函数
pub fn process_direct(context: &mut ReasonContextDirect) {
    let task_punctuation = context.current_task.get_().punctuation();

    // * 🚩根据语句类型（标点）分派推理
    use Punctuation::*;
    match task_punctuation {
        Judgement => process_judgement(context),
        Question => process_question(context),
    }
}

/// 接收判断作为信念
///
/// # 📄OpenNARS
///
/// To accept a new judgment as isBelief, and check for revisions and solutions
fn process_judgement(context: &mut ReasonContextDirect) {
    // * 🚩断言所基于的「当前概念」就是「推理上下文」的「当前概念」
    // * 📝在其被唯一使用的地方，传入的`task`只有可能是`context.currentConcept`
    let this = context.current_concept();
    // * 📝【2024-05-18 14:32:20】根据上游调用，此处「传入」的`task`只可能是`context.currentTask`
    let task = &context.current_task;
    // * 🚩断言传入任务的「语句」一定是「判断」
    debug_assert!(task.get_().is_judgement());
    let judgment = task.get_().as_judgement().unwrap().clone(); // ? 此处是否要将「任务」直接作为「信念」存储

    // * 🚩找到旧信念，并尝试修正
    let old_belief = evaluation(
        &judgment,
        this.beliefs().iter(),
        BudgetValue::solution_quality,
    );
    if let Some(old_belief) = old_belief {
        if judgment.evidential_eq(old_belief) {
            // * 🚩时间戳上重复⇒优先级沉底，避免重复推理
            let task = task.get_(); // 获取不可变引用，然后覆盖之
            if let Some(parent) = task.parent_task() {
                if parent.get_().is_judgement() {
                    drop(task); // 需要消除借用
                    let mut mut_task = context.current_task_mut();
                    mut_task.mut_().set_priority(ShortFloat::ZERO);
                    // duplicated task
                } // else: activated belief
                return;
            }
        }
        // * 🚩不重复 && 可修正 ⇒ 修正
        else if judgment.revisable_to(old_belief) {
            let has_overlap = judgment.evidential_overlap(old_belief);
            // * 🚩现在将「当前信念」「新时间戳」移入「修正」调用中
            if !has_overlap {
                // * 📌【2024-06-07 11:38:02】现在由于「新时间戳」的内置，经检查不再需要设置「当前信念」
                // * 📌此处的「当前信念」直接取`oldBelief`，并以此构造时间戳
                revision_direct(judgment.clone(), old_belief.clone(), context);
            }
        }
    }

    // * 🚩尝试用新的信念解决旧有问题
    // * 📄如：先输入`A?`再输入`A.`
    let budget_threshold = context.parameters().budget_threshold; // ! 需要单独分开：借用问题
    if context
        .current_task // ! 不能复用上头的task：可能会有借用问题
        .get_()
        .budget_above_threshold(budget_threshold)
    {
        // * 🚩开始尝试解决「问题表」中的所有问题
        let this = context.core.current_concept_mut();
        let mut outputs = vec![];
        let mut new_tasks = vec![];
        let mut results = vec![];
        // * 🚩先计算
        for existed_question in this.questions().iter() {
            let result = try_solution_calculate(&judgment, existed_question, budget_threshold);
            results.push(result);
        }
        // * 🚩再应用
        for (existed_question, result) in this.questions().iter_mut().zip(results.into_iter()) {
            // TODO: 🏗️有待重构：此处「应用修改需要激活任务，但激活任务需要借用上下文」存在严重借用问题
            let output = try_solution_apply(
                result,
                existed_question,
                &judgment,
                |new_budget, new_task, candidate_belief| {
                    {
                        let parent_task = context.current_task.clone(); // TODO: 原先要借用context的部分
                        let task = Task::new(
                            new_task.clone().into(),
                            new_budget,
                            Some(parent_task),
                            Some(new_task.clone()),
                            Some(candidate_belief.clone()),
                        );
                        // * 🚩现在重新改为`COMMENT`，但更详细地展示「任务」本身
                        {
                            let message = format!("!!! Activated: {}", task.to_display_long());
                            {
                                let output = util_outputs::output_comment(message);
                                outputs.push(output) // TODO: 原先要借用context的部分
                            };
                        };
                        // // * 🚩若为「问题」⇒输出显著的「导出结论」
                        new_tasks.push(task); // TODO: 原先要借用context的部分
                    }
                },
            );
            if let Some(output) = output {
                outputs.push(output);
            }
        }
        for output in outputs {
            context.report(output);
        }
        for new_task in new_tasks {
            context.add_new_task(new_task);
        }
        // TODO: 🏗️【2024-06-30 12:09:13】以上均为内联的代码
        // * 🚩将信念追加至「信念表」
        let this = context.core.current_concept_mut();
        let overflowed_belief = this.add_belief(judgment);
        // * 🚩报告溢出
        if let Some(overflowed_belief) = overflowed_belief {
            let output = util_outputs::output_comment(format!(
                "!!! Overflowed Belief in '{}': {}",
                this.term(),
                overflowed_belief.to_display_long()
            ));
            context.report(output);
        }
    }
}

/// 用已知信念回答问题
///
/// # 📄OpenNARS
///
/// To answer a question by existing beliefs
fn process_question(context: &mut ReasonContextDirect) {
    todo!()
}

/// 信念修正 @ 直接推理
/// * 🚩【2024-06-30 10:55:06】目前直接传入两个信念的所有权，避免借用问题
fn revision_direct(
    new_belief: impl Judgement,
    old_belief: impl Judgement,
    context: &mut ReasonContextDirect,
) {
    todo!()
}

/// 寻找已知问题
fn find_existed_question<'c>(concept: &'c Concept, task_content: &Term) -> Option<&'c Task> {
    todo!()
}

/// 答问评估
fn evaluation<'a, S, J: 'a>(
    query: &S,
    list: impl IntoIterator<Item = &'a J>,
    solution_query: fn(&S, &J) -> ShortFloat,
) -> Option<&'a J>
where
    S: Sentence,
    J: Judgement,
{
    let list = list.into_iter();
    todo!()
}
