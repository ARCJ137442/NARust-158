//! 本地推理
//! * 🎯承载原先「直接推理」的部分
//! * 📝其中包含「修订规则」等

use crate::{
    control::{ContextDerivation, ReasonContext, ReasonContextDirect},
    entity::{BudgetValue, Concept, Judgement, Punctuation, RCTask, Sentence, ShortFloat, Stamp},
    inference::{
        try_solution_apply_context, try_solution_apply_task, try_solution_calculate, Budget,
        BudgetFunctions, BudgetInference, Evidential, TruthFunctions,
    },
    language::Term,
    util::{RefCount, ToDisplayAndBrief},
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
    let old_belief = evaluation(&judgment, this.beliefs(), BudgetValue::solution_quality);
    if let Some((old_belief, ..)) = old_belief {
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
                revision_direct(context, judgment.clone(), old_belief.clone());
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
        let mut results = vec![];
        // * 🚩先计算
        for existed_question in this.questions() {
            let result =
                try_solution_calculate(&judgment, &existed_question.get_(), budget_threshold);
            // 拷贝「问题」的共享引用
            results.push((existed_question.clone_(), result));
        }
        // * 🚩再应用
        // * 📌【2024-07-09 14:56:32】现在分布应用，不再需要展开内联代码
        for (mut existed_question, result) in results {
            try_solution_apply_task(&result, &mut existed_question.mut_(), &judgment);
            try_solution_apply_context(result, &judgment, context);
        }
        // * 🚩将信念追加至「信念表」
        let this = context.current_concept_mut();
        let overflowed_belief = this.add_belief(judgment);
        // * 🚩报告溢出
        if let Some(overflowed_belief) = overflowed_belief {
            let message = format!(
                "!!! Overflowed Belief in '{}': {}",
                this.term(),
                overflowed_belief.to_display_long()
            );
            context.report_comment(message);
        }
    }
}

/// 用已知信念回答问题
///
/// # 📄OpenNARS
///
/// To answer a question by existing beliefs
fn process_question(context: &mut ReasonContextDirect) {
    let budget_threshold = context.parameters().budget_threshold;
    // * 📝【2024-05-18 14:32:20】根据上游调用，此处「传入」的`task`只可能是`context.currentTask`
    let question_task = context.current_task.clone_(); // * 🚩引用拷贝，否则会涉及大量借用问题
    let question_task_ref = question_task.get_(); // * 🚩引用拷贝，否则会涉及大量借用问题

    // * 🚩断言传入任务的「语句」一定是「问题」
    debug_assert!(
        question_task_ref.is_question(),
        "要处理的必须是「问题」：{question_task:?}"
    );

    // * 🚩断言所基于的「当前概念」就是「推理上下文」的「当前概念」
    // * 📝在其被唯一使用的地方，传入的`task`只有可能是`context.currentConcept`
    let this = context.current_concept();

    // * 🚩尝试寻找已有问题，若已有相同问题则直接处理已有问题
    let existed_question = find_existed_question(this, question_task_ref.content());
    let is_new_question = existed_question.is_none();
    let mut question = existed_question.unwrap_or(&question_task).clone_(); // ! 拷贝以避免借用问题

    // * 🚩实际上「先找答案，再新增『问题任务』」区别不大——找答案的时候，不会用到「问题任务」
    let new_answer = evaluation(
        &*question.get_(),
        this.beliefs(),
        BudgetValue::solution_quality,
    );
    if let Some((answer, ..)) = new_answer {
        let answer = answer.clone(); // ! 拷贝判断句以避免借用问题
        let result = try_solution_calculate(&answer, &question.get_(), budget_threshold);
        drop(question_task_ref);
        try_solution_apply_task(&result, &mut question.mut_(), &answer);
        try_solution_apply_context(result, &answer, context);
    } else {
        drop(question_task_ref);
    }
    // * 🚩新增问题
    if is_new_question {
        let overflowed_question = context.current_concept_mut().add_question(question_task);
        if let Some(task) = overflowed_question {
            context.report_comment(format!(
                "!!! Overflowed Question Task: {}",
                task.get_().to_display_long()
            ));
        }
    }
}

/// 信念修正 @ 直接推理
/// * 🚩【2024-06-30 10:55:06】目前直接传入两个信念的所有权，避免借用问题
fn revision_direct(
    context: &mut ReasonContextDirect,
    new_belief: impl Judgement,
    old_belief: impl Judgement,
) {
    // * 🚩词项
    let new_content = new_belief.clone_content();
    // * 🚩真值
    let new_truth = new_belief.revision(&old_belief);
    // * 🚩预算值
    let new_budget = BudgetValue::revise_direct(
        &new_belief,
        &old_belief,
        &new_truth,
        &mut *context.current_task.mut_(),
    );
    // * 🚩创建并导入结果：双前提
    // * 📝仅在此处用到「当前信念」作为「导出信念」
    // * 📝此处用不到「当前信念」（旧信念）
    // * 🚩【2024-06-06 08:52:56】现场构建「新时间戳」
    let new_stamp = Stamp::from_merge_unchecked(
        &new_belief,
        &old_belief,
        context.time(),
        context.max_evidence_base_length(),
    );
    context.double_premise_task_revision(new_content, new_truth, new_budget, new_stamp);
}

/// 根据输入的任务，寻找并尝试返回已有的问题
/// * ⚠️输出可空，且此时具有含义：概念中并没有「已有问题」
/// * 🚩经上游确认，此处的`task`只可能是`context.currentTask`
fn find_existed_question<'c>(concept: &'c Concept, task_content: &Term) -> Option<&'c RCTask> {
    // // * 🚩遍历所有已知问题：任意一个问题「词项相等」就返回
    // for existed_question in concept.questions().iter() {
    //     let question_term = existed_question.get_().content();
    //     // * 🚩词项相等⇒返回
    //     if question_term == task_content {
    //         return Some(existed_question);
    //     }
    // }
    // None;
    concept
        // * 🚩遍历所有已知问题：任意一个问题「词项相等」就返回
        .questions()
        .find(
            // * 🚩词项相等⇒返回
            |question| question.get_().content() == task_content,
        )
}

/// 答问评估
/// * ✨增加功能：返回包括「解答质量」在内的整个结果
///
/// # 📄OpenNARS
///
/// Evaluate a query against beliefs (and desires in the future)
fn evaluation<'a, S, J: 'a>(
    query: &S,
    list: impl IntoIterator<Item = &'a J>,
    solution_quality: fn(&S, &J) -> ShortFloat,
) -> Option<(&'a J, ShortFloat)>
where
    S: Sentence,
    J: Judgement,
{
    // * 🚩筛选出其中排行最前的回答
    let mut current_best = ShortFloat::default();
    let mut candidate = None;
    for judgement in list {
        let belief_quality = solution_quality(query, judgement);
        // * 🚩排行大于⇒更新
        if belief_quality > current_best {
            current_best = belief_quality;
            candidate = Some(judgement);
        }
    }
    // * 🚩将最大值也一并传出
    candidate.map(|solution| (solution, current_best))
    // ! ❌【2024-07-02 16:43:44】不能使用迭代器方法
    // * 📝在处理「等号情况」时，`max_by_key`要用后者【覆盖】前者
    // * 测试代码：`dbg!([-1_i32, 1, 2, 3, -3, -2, 0].iter().max_by_key(|n| n.abs()));`返回`-3`而非`3`
    // list.into_iter().max_by_key(|judgement| solution_query(query, judgement))
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::{test::*, InferenceEngine};
    use navm::output::Output;

    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct,
        InferenceEngine::ECHO.transform_f(),
        InferenceEngine::ECHO.matching_f(),
        InferenceEngine::ECHO.reason_f(),
    );

    /// 直接回答问题
    #[test]
    fn direct_answer_question() {
        let mut vm = create_vm_from_engine(ENGINE);
        // * 🚩输入指令并拉取输出
        let outs = vm.input_cmds_and_fetch_out(
            "
            nse Sentence.
            nse Sentence?
            cyc 2
            ",
        );
        // * 🚩打印输出
        print_outputs(&outs);
        // * 🚩检查其中是否有回答
        expect_outputs(&outs, |answer| matches!(answer, Output::ANSWER { .. }));
    }

    /// 稳定性
    #[test]
    fn stability() {
        let mut vm = create_vm_from_engine(ENGINE);
        // * 🚩检验长期稳定性
        for i in 0..0x100 {
            let _outs = vm.input_cmds_and_fetch_out(&format!(
                "
                    nse <A{i} --> B>.
                    nse <A{i} --> B>?
                    rem cyc 50
                    "
            ));
            // ! ⚠️【2024-07-09 02:22:12】不一定有回答：预算竞争约束着资源调配，可能没法立即回答
            // // * 🚩检测有回答
            // expect_outputs(&outs, |answer| matches!(answer, Output::ANSWER { .. }));
        }
        vm.input_cmds("cyc 10000");
    }
}
