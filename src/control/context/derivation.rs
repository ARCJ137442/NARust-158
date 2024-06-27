//! 有关「推理上下文」中「导出结论」的功能
//! * 🎯分离并锁定「导出结论」的方法

use narsese::api::NarseseValue;
use navm::output::Output;

use super::{ReasonContext, ReasonContextConcept, ReasonContextDirect};
use crate::{
    entity::{BudgetValue, JudgementV1, Sentence, SentenceV1, Stamp, Task, TaskLink, TruthValue},
    inference::Budget,
    language::Term,
    util::{RefCount, ToDisplayAndBrief},
};

/// 自动实现 for 「推理上下文」
pub trait ContextDerivation: ReasonContext {
    // TODO: 将以下逻辑迁移到单独的「自动实现之特征」中
    /// 共用终端逻辑：「激活任务」
    /// # 📄OpenNARS
    ///
    /// Activated task called in MatchingRules.trySolution and
    /// Concept.processGoal
    /// * 📝仅被「答问」调用
    fn activated_task(
        &mut self,
        new_budget: BudgetValue,
        new_task: &JudgementV1,
        candidate_belief: &JudgementV1,
    ) {
        let task = Task::new(
            SentenceV1::JudgementV1(new_task.clone()),
            new_budget,
            Some(self.current_task().clone()),
            Some(new_task.clone()),
            Some(candidate_belief.clone()),
        );
        // * 🚩现在重新改为`COMMENT`，但更详细地展示「任务」本身
        self.add_output(Output::COMMENT {
            content: format!("!!! Activated: {}", task.to_display_long()),
        });
        // // * 🚩若为「问题」⇒输出显著的「导出结论」
        self.add_new_task(task);
    }

    /// 共用终端逻辑：「导出任务」
    ///
    /// # 📄OpenNARS
    ///
    /// Derived task comes from the inference rules.
    fn derived_task(&mut self, new_task: Task) {
        // * 🚩判断「导出的新任务」是否有价值
        if !new_task.budget_above_threshold(self.parameters().budget_threshold) {
            self.add_output(Output::COMMENT {
                content: format!("!!! Ignored: {}", new_task.to_display_long()),
            });
            return;
        }
        // * 🚩报告
        self.add_output(Output::COMMENT {
            content: format!("!!! Derived: {}", new_task.to_display_long()),
        });
        let budget_summary = new_task.budget_summary().to_float();
        if budget_summary > self.silence_percent() {
            // only report significant derived Tasks
            let narsese = Some(NarseseValue::Task(new_task.to_lexical()));
            self.add_output(Output::OUT {
                content_raw: format!("OUT: {}", new_task.to_display_long()),
                narsese,
            });
        }
        // * 🚩将「导出的新任务」添加到「新任务表」中
        self.add_new_task(new_task);
    }

    /// 🆕仅源自「修正规则」调用，没有「父信念」
    fn double_premise_task_revision(
        &mut self,
        new_content: Term,
        new_truth: TruthValue,
        new_budget: BudgetValue,
        new_stamp: Stamp,
    ) {
        // * 🚩仅在「任务内容」可用时构造
        let current_task = self.current_task().get_(); // 不能当场变为引用：后续可能要再借用自身
        let new_punctuation = current_task.punctuation();
        let new_sentence = SentenceV1::new_sentence_from_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            Some((new_truth, true)),
        );
        drop(current_task); // ! 先抛掉引用代理
        match new_sentence {
            Ok(new_sentence) => {
                let new_task = Task::new(
                    new_sentence,
                    new_budget,
                    Some(self.current_task().clone()),
                    None,
                    None,
                );
                self.derived_task(new_task);
            }
            Err(error) => self.add_output(Output::ERROR {
                description: error.to_string(),
            }),
        }
    }
}

/// 对「所有实现了『推理上下文』的结构」实现该特征
/// * 📝需要采用`?Sized`以包括【运行时尺寸未定】的对象
///   * ⚠️不然默认仅对[`Sized`]实现
impl<T: ?Sized + ReasonContext> ContextDerivation for T {}

pub trait ContextDerivationConcept: ReasonContextConcept {
    // TODO: 统一迁移到别的模块
    /// 🆕产生新时间戳 from 单前提
    fn generate_new_stamp_single(&self) -> Stamp {
        let current_task = self.current_task().get_();
        match (current_task.is_judgement(), self.current_belief()) {
            // * 🚩「当前任务」是判断句 | 没有「当前信念」
            (true, _) | (_, None) => Stamp::with_old(&*current_task, self.time()),
            // * 🚩其它 ⇒ 时间戳来自信念
            // to answer a question with negation in NAL-5 --- move to activated task?
            (false, Some(belief)) => Stamp::with_old(belief, self.time()),
        }
    }

    /// 🆕产生新时间戳 from 双前提
    fn generate_new_stamp_double(&self) -> Option<Stamp> {
        let current_task = &*self.current_task().get_();
        // * 🚩在具有「当前信念」时返回「与『当前任务』合并的时间戳」
        self.current_belief().map(|belief|
                // * 📌此处的「时间戳」一定是「当前信念」的时间戳
                // * 📄理由：最后返回的信念与「成功时比对的信念」一致（只隔着`clone`）
                 Stamp::from_merge_unchecked(current_task, belief, self.time(), self.max_evidence_base_length()))
    }

    /// Shared final operations by all double-premise rules, called from the
    /// rules except StructuralRules
    /// * 🚩【2024-05-19 12:44:55】构造函数简化：导出的结论【始终可修正】
    fn double_premise_task(
        &mut self,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: BudgetValue,
    ) {
        // * 🚩尝试创建「新时间戳」然后使用之
        if let Some(new_stamp) = self.generate_new_stamp_double() {
            let new_truth_revisable = new_truth.map(|truth| (truth, true));
            self.double_premise_task_full(
                None,
                new_content,
                new_truth_revisable,
                new_budget,
                new_stamp,
            )
        }
    }

    /// 🆕其直接调用来自组合规则、匹配规则（修正）
    /// * 🎯避免对`currentTask`的赋值，解耦调用（并让`currentTask`不可变）
    /// * 🎯避免对`newStamp`的复制，解耦调用（让「新时间戳」的赋值止步在「推理开始」之前）
    /// * 🚩【2024-06-27 00:52:39】为避免借用冲突，此处使用[`Option`]区分「传入其它地方引用/使用自身引用」
    ///   * 有值 ⇒ 使用内部的值
    ///   * 空值 ⇒ 从`self`中拿取
    fn double_premise_task_full(
        &mut self,
        current_task: Option<&Task>,
        new_content: Term,
        new_truth_revisable: Option<(TruthValue, bool)>,
        new_budget: BudgetValue,
        new_stamp: Stamp,
    ) {
        // * 🚩参考「传入任务/自身默认任务」构造标点
        let new_punctuation = current_task
            .unwrap_or(&*self.current_task().get_()) // 立即使用的不可变引用
            .punctuation();
        let new_sentence = SentenceV1::new_sentence_from_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            new_truth_revisable,
        );
        if let Ok(sentence) = new_sentence {
            let new_task = Task::from_derived(
                sentence,
                new_budget,
                Some(self.current_task().clone()),
                self.current_belief().cloned(),
            );
            // * 🚩正式导出结论（在这之前注销代理）
            self.derived_task(new_task);
        }
    }

    /// 🆕重定向
    fn double_premise_task_not_revisable(
        &mut self,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: BudgetValue,
    ) {
        todo!("【2024-06-27 01:10:54】后续再弄")
    }

    //     /// Shared final operations by all double-premise rules,
    // /// called from the rules except StructuralRules
    // double_premise_task_
}

/// * 📝需要采用`?Sized`以包括【运行时尺寸未定】的对象
///   * ⚠️不然默认仅对[`Sized`]实现
impl<T: ?Sized + ReasonContextConcept> ContextDerivationConcept for T {}
