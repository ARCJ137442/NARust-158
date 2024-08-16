//! 有关「推理上下文」中「导出结论」的功能
//! * 🎯分离并锁定「导出结论」的方法

use super::{ReasonContext, ReasonContextWithLinks};
use crate::{
    entity::{
        BudgetValue, Judgement, JudgementV1, Punctuation, Sentence, SentenceV1, Stamp, Task,
        TruthValue,
    },
    inference::Budget,
    language::Term,
    util::{RefCount, ToDisplayAndBrief},
};

/// 自动实现 for 「推理上下文」
pub trait ContextDerivation: ReasonContext {
    /// 共用终端逻辑：「激活任务」
    /// # 📄OpenNARS
    ///
    /// Activated task called in MatchingRules.trySolution and
    /// Concept.processGoal
    /// * 📝仅被「答问」调用
    fn activated_task(
        &mut self,
        new_budget: impl Into<BudgetValue>,
        solution: &JudgementV1,
        candidate_belief: &JudgementV1,
    ) {
        let parent_task = self.current_task().clone();
        let task = Task::new(
            solution.clone().into(),
            new_budget.into(),
            Some(parent_task),
            Some(solution.clone()),
            Some(candidate_belief.clone()),
        );
        // * 🚩现在重新改为`COMMENT`，但更详细地展示「任务」本身
        self.report_comment(format!("!!! Activated: {}", task.to_display_long()));
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
            self.report_comment(format!("!!! Ignored: {}", new_task.to_display_long()));
            return;
        }
        // * 🚩报告
        self.report_comment(format!("!!! Derived: {}", new_task.to_display_long()));
        let budget_summary = new_task.budget_summary().to_float();
        if budget_summary > self.silence_percent() {
            // only report significant derived Tasks
            self.report_out(&new_task);
        }
        // * 🚩将「导出的新任务」添加到「新任务表」中
        self.add_new_task(new_task);
    }

    /// 🆕仅源自「修正规则」调用，没有「父信念」
    fn double_premise_task_revision(
        &mut self,
        new_content: Term,
        new_truth: impl Into<TruthValue>,
        new_budget: impl Into<BudgetValue>,
        new_stamp: Stamp,
    ) {
        // * 🚩仅在「任务内容」可用时构造
        let current_task = self.current_task(); // 不能当场变为引用：后续可能要再借用自身
        let new_punctuation = current_task.get_().punctuation();
        let new_sentence = SentenceV1::with_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            Some((new_truth.into(), true)),
        );
        drop(current_task); // ! 先抛掉引用代理
        match new_sentence {
            Ok(new_sentence) => {
                let new_task = Task::new(
                    new_sentence,
                    new_budget.into(),
                    Some(self.current_task().clone()),
                    None,
                    None,
                );
                self.derived_task(new_task);
            }
            Err(error) => self.report_error(error.to_string()),
        }
    }
}

/// 对「所有实现了『推理上下文』的结构」实现该特征
/// * 📝需要采用`?Sized`以包括【运行时尺寸未定】的对象
///   * ⚠️不然默认仅对[`Sized`]实现
impl<T: ?Sized + ReasonContext> ContextDerivation for T {}

pub trait ContextDerivationConcept: ReasonContextWithLinks {
    /// 🆕产生新时间戳 from 单前提
    fn generate_new_stamp_single(&self) -> Stamp {
        let current_task_ref = self.current_task();
        let current_task = current_task_ref.get_();
        match (current_task.is_judgement(), self.current_belief()) {
            // * 🚩「当前任务」是判断句 | 没有「当前信念」
            (true, _) | (_, None) => Stamp::with_old(&*current_task, self.time()),
            // * 🚩其它 ⇒ 时间戳来自信念
            // to answer a question with negation in NAL-5 --- move to activated task?
            (false, Some(belief)) => Stamp::with_old(belief, self.time()),
        }
    }

    /// 🆕产生新时间戳 from 双前提
    ///
    /// ? 是否需要通过「假定有『当前信念』」实现「直接返回[`Stamp`]而非[`Option<Stamp>`](Option)」？
    fn generate_new_stamp_double(&self) -> Option<Stamp> {
        let current_task_ref = self.current_task();
        let current_task = current_task_ref.get_();
        // * 🚩在具有「当前信念」时返回「与『当前任务』合并的时间戳」
        self.current_belief().map(|belief|
                // * 📌此处的「时间戳」一定是「当前信念」的时间戳
                // * 📄理由：最后返回的信念与「成功时比对的信念」一致（只隔着`clone`）
                 Stamp::from_merge_unchecked(&*current_task, belief, self.time(), self.max_evidence_base_length()))
    }

    /* --------------- new task building --------------- */

    /// Shared final operations by all double-premise rules, called from the
    /// rules except StructuralRules
    /// * 🚩【2024-05-19 12:44:55】构造函数简化：导出的结论【始终可修正】
    fn double_premise_task(
        &mut self,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: impl Into<BudgetValue>,
    ) {
        // * 🚩尝试创建「新时间戳」然后使用之
        if let Some(new_stamp) = self.generate_new_stamp_double() {
            let new_truth_revisable = new_truth.map(|truth| (truth, true));
            self.double_premise_task_full(
                None, // * 🚩默认「当前任务」
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
    fn double_premise_task_compositional(
        &mut self,
        current_task: &Task,
        new_content: Term,
        new_truth: Option<TruthValue>,
        new_budget: impl Into<BudgetValue>,
        new_stamp: Stamp,
    ) {
        self.double_premise_task_full(
            Some(current_task),
            new_content,
            // * 🚩默认「可修正」
            new_truth.map(|truth| (truth, true)),
            new_budget,
            new_stamp,
        )
    }

    /// 🆕重定向
    fn double_premise_task_not_revisable(
        &mut self,
        new_content: Term,
        new_truth: Option<impl Into<TruthValue>>,
        new_budget: impl Into<BudgetValue>,
    ) {
        if let Some(new_stamp) = self.generate_new_stamp_double() {
            self.double_premise_task_full(
                None, // * 🚩默认「当前任务」
                new_content,
                // * 🚩默认「不可修正」，其它相同
                new_truth.map(|truth| (truth, false)),
                new_budget,
                new_stamp,
            )
        }
    }

    /// 「双前提导出结论」的完整方法实现
    /// * 🚩【2024-06-27 00:52:39】为避免借用冲突，此处使用[`Option`]区分「传入其它地方引用/使用自身引用」
    ///   * 有值 ⇒ 使用内部的值
    ///   * 空值 ⇒ 从`self`中拿取
    ///
    /// # 📄OpenNARS
    ///
    /// Shared final operations by all double-premise rules,
    /// called from the rules except StructuralRules
    fn double_premise_task_full(
        &mut self,
        current_task: Option<&Task>,
        new_content: Term,
        new_truth_revisable: Option<(impl Into<TruthValue>, bool)>,
        new_budget: impl Into<BudgetValue>,
        new_stamp: Stamp,
    ) {
        // * 🚩参考「传入任务/自身默认任务」构造标点
        let new_punctuation = current_task
            .unwrap_or(&*self.current_task().get_()) // 立即使用的不可变引用
            .punctuation();
        let new_sentence = SentenceV1::with_punctuation(
            new_content,
            new_punctuation,
            new_stamp,
            new_truth_revisable.map(|(truth, revisable)| (truth.into(), revisable)),
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

    /// Shared final operations by all single-premise rules,
    /// called in StructuralRules
    fn single_premise_task_full(
        &mut self,
        new_content: Term,
        punctuation: Punctuation,
        new_truth: Option<impl Into<TruthValue>>,
        new_budget: impl Into<BudgetValue>,
    ) {
        // * 🚩兼容各类「真值」「预算值」的引用（自动转换成真值）
        let new_truth = new_truth.map(Into::into);
        let new_budget = new_budget.into();
        let current_task_ref = self.current_task();
        let current_task = current_task_ref.get_();
        let parent_task = current_task.parent_task();
        // * 🚩对于「结构转换」的单前提推理，若已有父任务且该任务与父任务相同⇒中止，避免重复推理
        if let Some(parent_task) = parent_task {
            if new_content == *parent_task.get_().content() {
                return; // to avoid circular structural inference
            }
        }
        let task_sentence = &*current_task;
        // * 🚩构造新时间戳
        let new_stamp = self.generate_new_stamp_single();
        // * 🚩使用新内容构造新语句
        let revisable = task_sentence
            .as_judgement()
            // * 🚩判断句⇒返回实际的「可修订」
            // * 🚩疑问句⇒返回一个用不到的空值
            .map_or(false, Judgement::revisable);
        // * 🚩判断句⇒返回实际的「可修订」
        // * 🚩疑问句⇒返回一个用不到的空值
        let new_sentence = SentenceV1::with_punctuation(
            new_content,
            punctuation,
            new_stamp,
            new_truth.map(|truth| (truth, revisable)),
        );
        let new_sentence = match new_sentence {
            // * 🚩伪·问号解包
            Ok(sentence) => sentence,
            Err(..) => return,
        };
        // * 🚩构造新任务
        let new_task = Task::from_derived(
            new_sentence,
            new_budget,
            // * 🚩拷贝共享引用
            Some(self.current_task().clone()),
            None,
        );
        // * 🚩导出
        drop(current_task); // ! 先释放「借用代理」
        drop(current_task_ref);
        self.derived_task(new_task);
    }

    /// 来自「结构规则」与「转换规则」的单前提导出
    /// * 🚩除了「标点」固定指向「当前任务」外，其它与[完整方法](ContextDerivationConcept::single_premise_task_full)一致
    fn single_premise_task_structural(
        &mut self,
        new_content: Term,
        new_truth: Option<impl Into<TruthValue>>,
        new_budget: impl Into<BudgetValue>,
    ) {
        // * 🚩新任务标点取自「当前任务」
        let punctuation = self.current_task().get_().punctuation();
        self.single_premise_task_full(new_content, punctuation, new_truth, new_budget)
    }
}

/// * 📝需要采用`?Sized`以包括【运行时尺寸未定】的对象
///   * ⚠️不然默认仅对[`Sized`]实现
impl<T: ?Sized + ReasonContextWithLinks> ContextDerivationConcept for T {}
