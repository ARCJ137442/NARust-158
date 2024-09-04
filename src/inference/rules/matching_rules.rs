//! 匹配规则
//! * ℹ️概念推理之前，先匹配「任务」与「信念」

use crate::{
    control::{
        ContextDerivationConcept, ReasonContext, ReasonContextConcept, ReasonContextWithLinks,
    },
    entity::{Judgement, PunctuatedSentenceRef, Sentence, Stamp, TruthValue},
    inference::{
        try_solution_apply_context, try_solution_apply_task, try_solution_calculate,
        BudgetInferenceContext, TruthFunctions,
    },
    language::{variable_process, Term},
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;

/// 匹配推理 入口函数
/// * 🚩【2024-06-28 17:23:54】目前作为「匹配推理」的入口，不再直接暴露在控制机制中
/// * 📝「匹配推理」的核心：拿到一个任务链，再拿到一个信念链，先直接在其中做匹配
/// * 📝「匹配推理」的作用：信念修正任务、信念回答「特殊疑问」
///
/// # 📄OpenNARS
///
/// The task and belief have the same content
pub fn match_task_and_belief(context: &mut ReasonContextConcept) {
    // * 🚩保证提取出「当前信念」
    let shuffle_rng_seed = context.shuffle_rng_seeds(); // 提前生成随机种子
    let current_belief = unwrap_or_return!(?context.current_belief());
    let current_task_rc = context.current_task();
    let current_task = current_task_rc.get_();
    let current_task_punctuation = current_task.as_punctuated_ref();

    // * 🚩按照标点分派
    use PunctuatedSentenceRef::*;
    match current_task_punctuation {
        // * 🚩判断⇒尝试修正
        Judgement(judgement) => {
            // * 🚩判断「当前任务」是否能与「当前信念」做修正
            if judgement.revisable_to(current_belief) {
                // 先复用不可变元素，推理导出结果
                let (content, truth, stamp) = revision(judgement, current_belief, context);
                // 计算预算值：需要修改上下文
                let [current_task_truth, current_belief_truth] = [
                    TruthValue::from(judgement),
                    TruthValue::from(current_belief),
                ]; // ! 防止借用问题
                drop(current_task);
                drop(current_task_rc);
                let budget =
                    context.revise_matching(&current_task_truth, &current_belief_truth, &truth);
                // * 🚩创建并导入结果：双前提 | 📝仅在此处用到「当前信念」作为「导出信念」
                context.double_premise_task_full(None, content, Some((truth, true)), budget, stamp);
            }
        }
        // * 🚩问题⇒尝试回答「特殊疑问」（此处用「变量替换」解决查询变量）
        Question(question) => {
            // * 📝只有「匹配已知」才能回答「特殊疑问」，「一般疑问」交由「直接推理」回答
            // * 🚩查看是否可以替换「查询变量」，具体替换从「特殊疑问」转变为「一般疑问」
            // * 📄Task :: SentenceV1@49 "<{?1} --> murder>? {105 : 6} "
            // * & Belief: SentenceV1@39 "<{tom} --> murder>. %1.0000;0.7290% {147 : 3;4;2}"
            // * ⇒ Unified SentenceV1@23 "<{tom} --> murder>? {105 : 6} "
            let has_unified = variable_process::has_unification_q(
                question.content(),
                current_belief.content(),
                shuffle_rng_seed,
            );
            // * ⚠️只针对「特殊疑问」：传入的只有「带变量问题」，因为「一般疑问」通过直接推理就完成了
            if has_unified {
                // * 🚩此时「当前任务」「当前信念」仍然没变
                // 计算
                let result = try_solution_calculate(
                    current_belief,
                    &current_task,
                    context.parameters().budget_threshold,
                );
                // 应用 @ 任务
                drop(current_task);
                drop(current_task_rc);
                let current_belief = current_belief.clone(); // ! 复制以防止借用冲突
                let mut current_task_rc = context.current_task_mut();
                let mut current_task = current_task_rc.mut_();
                try_solution_apply_task(&result, &mut current_task, &current_belief);
                // 应用 @ 上下文
                drop(current_task);
                drop(current_task_rc);
                try_solution_apply_context(result, &current_belief, context);
            }
        }
    }
}

/// 🆕基于「概念推理」的「修正」规则
/// * 📝和「直接推理」的唯一区别：有「当前信念」（会作为「父信念」使用 ）
/// * 💭【2024-06-09 01:35:41】需要合并逻辑
/// * ⚠️不能在此计算「预算值」，因为计算时要修改上下文
fn revision(
    new_belief: &impl Judgement,
    old_belief: &impl Judgement,
    context: &ReasonContextConcept,
) -> (Term, TruthValue, Stamp) {
    // * 🚩内容
    let content = new_belief.content().clone();
    // * 🚩计算真值
    let revised_truth = new_belief.revision(old_belief);
    // * 🚩【2024-06-06 08:52:56】现场构建「新时间戳」
    let new_stamp = Stamp::from_merge_unchecked(
        new_belief,
        old_belief,
        context.time(),
        context.max_evidence_base_length(),
    );
    // * 🚩返回
    (content, revised_truth, new_stamp)
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expect_narsese_term,
        inference::{process_direct, tools::*, InferenceEngine},
    };

    /// 引擎
    const ENGINE: InferenceEngine = InferenceEngine::new(
        process_direct, // ! 必要：需要内化成「信念」再进行匹配
        InferenceEngine::ECHO.transform_f(),
        match_task_and_belief,
        InferenceEngine::ECHO.reason_f(),
    );

    /// 修正判断
    #[test]
    fn revise_after_direct() {
        let mut vm = create_reasoner_from_engine(ENGINE);
        // * 🚩输入指令并拉取输出
        vm.input_fetch_print_expect(
            "
            nse Sentence. %1.0;0.5%
            cyc 5
            nse Sentence. %0.0;0.5%
            cyc 5
            ",
            // * 🚩检查其中是否有导出
            expect_narsese_term!(OUT "Sentence" in outputs),
        );
    }

    /// 修正判断+答问
    #[test]
    fn answer_after_revise() {
        let mut vm = create_reasoner_from_engine(ENGINE);

        // 匹配时回答
        vm.input_fetch_print_expect(
            "
            nse Sentence. %1.0;0.5%
            cyc 2
            nse Sentence?
            cyc 2
            ",
            expect_narsese_term!(ANSWER "Sentence" in outputs),
        );

        // 修正后回答
        vm.input_fetch_print_expect(
            "
            nse Sentence. %0.0;0.5%
            cyc 2
            ",
            expect_narsese_term!(ANSWER "Sentence" in outputs),
        );

        // 修正后回答
        vm.input_fetch_print_expect(
            "
            nse Sentence. %0.5;0.5%
            cyc 2
            ",
            expect_narsese_term!(ANSWER "Sentence" in outputs),
        );
    }

    /// 回答带变量问题
    #[test]
    fn answer_question_with_variables() {
        let mut vm = create_reasoner_from_engine(ENGINE);
        // * 🚩输入指令并拉取输出
        vm.input_fetch_print_expect(
            "
            nse <A --> B>.
            cyc 5
            nse <?1 --> B>?
            cyc 50
            ",
            expect_narsese_term!(ANSWER "<A --> B>" in outputs),
        );
        vm.input_fetch_print_expect(
            "
            res
            nse <A --> B>.
            cyc 5
            nse <A --> ?1>?
            cyc 50
            ",
            expect_narsese_term!(ANSWER "<A --> B>" in outputs),
        );
    }

    /// 稳定性
    /// * 🚩【2024-08-12 22:56:38】考虑到单测时间太长，目前压到16轮
    #[test]
    fn stability() {
        let mut vm = create_reasoner_from_engine(ENGINE);
        // * 🚩检验长期稳定性
        for i in 0..0x10 {
            let _outs = vm.input_cmds_and_fetch_out(&format!(
                "
                nse <A{i} --> B>. %1.0;0.9%
                cyc 5
                nse <A{i} --> B>. %0.0;0.9%
                cyc 5
                nse <A{i} --> B>?
                cyc 5
                "
            ));
            // ! ⚠️【2024-07-09 02:22:12】不一定有回答：预算竞争约束着资源调配，可能没法立即回答
            // // * 🚩检测有回答
            // expect_outputs(&outs, |answer| matches!(answer, Output::ANSWER { .. }));
        }
        vm.input_cmds("cyc 1000");
    }
}
