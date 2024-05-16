//! 基于「推导上下文」对「记忆区」有关「导出结论」的操作
//! * 🎯将其中有关「导出结论」的代码摘录出来
//!   * 📌任务激活 from 本地规则（问答）
//!   * 📌导出任务(内部) from 单前提结论、双前提结论
//!   * 📌双前提结论(可修正) from 组合规则、本地规则、三段论规则
//!   * 📌双前提结论 from 组合规则
//!   * 📌单前提结论(当前任务之标点) from 结构规则
//!   * 📌单前提结论 from 本地规则、结构规则
//! * 📝该处逻辑均在OpenNARS中用作「产生（并存储）导出结论」
//!   TODO: 后续或考虑完全基于「推导上下文」
//!
//! * ✅【2024-05-12 16:10:24】基本迁移完所有功能

use crate::{entity::*, inference::*, language::Term, nars::DEFAULT_PARAMETERS, *};
use narsese::api::NarseseValue;
use navm::output::Output;

/// 记忆区处理：整理与「记忆区」有关的操作
/// * 🚩目前以「记忆区」为中心，以便从「记忆区」处添加方法
/// * 🚩【2024-05-12 15:00:59】因为`RuleTables::transform_task(self);`，要求[`Sized`]
pub trait MemoryDerivationProcess: DerivationContext {
    /// 模拟`Memory.activatedTask`
    /// * 📝OpenNARS中仅用于「本地规则」
    ///
    /// # 📄OpenNARS
    ///
    /// Activated task called in MatchingRules.trySolution and Concept.processGoal
    ///
    /// @param budget          The budget value of the new Task
    /// @param sentence        The content of the new Task
    /// @param candidateBelief The belief to be used in future inference, for forward/backward correspondence
    fn activated_task(
        &mut self,
        budget: &Self::Budget,
        sentence: Self::Sentence,
        candidate_belief: Self::Sentence,
    ) {
        /* 📄OpenNARS源码：
        Task task = new Task(sentence, budget, currentTask, sentence, candidateBelief);
        recorder.append("!!! Activated: " + task.toString() + "\n");
        if (sentence.isQuestion()) {
            float s = task.getBudget().summary();
            // float minSilent = reasoner.getMainWindow().silentW.value() / 100.0f;
            float minSilent = reasoner.getSilenceValue().get() / 100.0f;
            if (s > minSilent) { // only report significant derived Tasks
                report(task.getSentence(), ReportType.OUT);
            }
        }
        newTasks.add(task); */
        let task = <Self::Task as TaskConcrete>::from_activate(
            sentence.clone(),
            budget.clone(),
            self.current_task().clone(),
            sentence.clone(),
            candidate_belief,
        );
        // * 🚩现在重新改为`COMMENT`，但更详细地展示「任务」本身
        self.report(Output::COMMENT {
            content: format!("!!! Activated: {}", task.to_display_long()),
        });
        // 问题⇒尝试输出
        // * 🚩决议：有关「静音音量」的问题，交由「记忆区」而非「推导上下文」决定
        if let SentenceType::Question = sentence.punctuation() {
            let s = task.budget().summary().to_float();
            if s > self.silence_percent() {
                let narsese = NarseseValue::from_task(task.to_lexical());
                self.report(Output::OUT {
                    content_raw: format!("!!! Derived: {}", task.to_display()),
                    narsese: Some(narsese),
                });
            }
        }
        // 追加到「推导上下文」的「新任务」
        self.__new_tasks_mut().push(task);
    }

    /// 模拟`Memory.derivedTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Derived task comes from the inference rules.
    ///
    /// @param task the derived task
    fn derived_task(&mut self, task: Self::Task) {
        /* 📄OpenNARS源码：
        if (task.getBudget().aboveThreshold()) {
            recorder.append("!!! Derived: " + task + "\n");
            float budget = task.getBudget().summary();
            // float minSilent = reasoner.getMainWindow().silentW.value() / 100.0f;
            float minSilent = reasoner.getSilenceValue().get() / 100.0f;
            if (budget > minSilent) { // only report significant derived Tasks
                report(task.getSentence(), ReportType.OUT);
            }
            newTasks.add(task);
        } else {
            recorder.append("!!! Ignored: " + task + "\n");
        } */
        let budget_threshold = DEFAULT_PARAMETERS.budget_threshold;
        let budget_threshold = Self::ShortFloat::from_float(budget_threshold);
        let budget_summary = task.summary().to_float();
        // * 🚩🆕【2024-05-08 14:45:59】合并条件：预算值在阈值之上 && 达到（日志用的）音量水平
        if task.above_threshold(budget_threshold) && budget_summary > self.silence_percent() {
            self.report(Output::OUT {
                content_raw: format!("!!! Derived: {}", task.content()),
                narsese: Some(NarseseValue::from_task(task.to_lexical())),
            });
            self.__new_tasks_mut().push(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.report(Output::COMMENT {
                content: format!("!!! Ignored: {}", task.to_display_long()),
            });
        }
    }

    /* --------------- new task building --------------- */

    /// 模拟`Memory.doublePremiseTask`
    /// * ✅此处无需判断「新内容」为空：编译期非空检查
    /// * ⚠️需要保证自身「新时间戳」非空
    ///
    /// # 📄OpenNARS
    ///
    /// Shared final operations by all double-premise rules, called from the
    /// rules except StructuralRules
    ///
    /// @param newContent The content of the sentence in task
    /// @param newTruth   The truth value of the sentence in task
    /// @param newBudget  The budget value in task
    fn double_premise_task_revisable(
        &mut self,
        new_content: Term,
        new_truth: Self::Truth,
        new_budget: Self::Budget,
    ) {
        /* 📄OpenNARS源码：
        if (newContent != null) {
            Sentence newSentence = new Sentence(newContent, currentTask.getSentence().getPunctuation(), newTruth, newStamp);
            Task newTask = new Task(newSentence, newBudget, currentTask, currentBelief);
            derivedTask(newTask);
        } */
        let mut new_punctuation = self.current_task().sentence().punctuation().clone();
        // * 🆕🚩【2024-05-08 11:52:03】需要以此将「真值」插入「语句类型/标点」中（「问题」可能没有真值）
        if let SentenceType::Judgement(truth) = &mut new_punctuation {
            *truth = new_truth;
        }
        let new_sentence = <Self::Sentence as SentenceConcrete>::new_revisable(
            new_content,
            new_punctuation,
            self.new_stamp().as_ref().unwrap().clone(),
        );
        let new_task = <Self::Task as TaskConcrete>::from_derive(
            new_sentence,
            new_budget,
            Some(self.current_task().clone()),
            self.current_belief().clone(),
        );
        self.derived_task(new_task);
    }

    /// 模拟`Memory.doublePremiseTask`
    /// * 📌【2024-05-08 11:57:38】相比[`Memory::double_premise_task_revisable`]多了个`revisable`作为「语句」的推理参数
    ///   * 🚩作用在「语句」上
    /// * ⚠️要求`new_stamp`字段非空
    ///
    /// # 📄OpenNARS
    ///
    /// Shared final operations by all double-premise rules, called from the
    /// rules except StructuralRules
    ///
    /// @param newContent The content of the sentence in task
    /// @param newTruth   The truth value of the sentence in task
    /// @param newBudget  The budget value in task
    /// @param revisable  Whether the sentence is revisable
    fn double_premise_task(
        &mut self,
        new_content: Term,
        new_truth: Self::Truth,
        new_budget: Self::Budget,
        revisable: bool,
    ) {
        /* 📄OpenNARS源码：
        if (newContent != null) {
            Sentence taskSentence = currentTask.getSentence();
            Sentence newSentence = new Sentence(newContent, taskSentence.getPunctuation(), newTruth, newStamp,
                    revisable);
            Task newTask = new Task(newSentence, newBudget, currentTask, currentBelief);
            derivedTask(newTask);
        } */
        let mut new_punctuation = self.current_task().sentence().punctuation().clone();
        // * 🆕🚩【2024-05-08 11:52:03】需要以此将「真值」插入「语句类型/标点」中（「问题」可能没有真值）
        if let SentenceType::Judgement(truth) = &mut new_punctuation {
            *truth = new_truth;
        }
        let new_sentence = <Self::Sentence as SentenceConcrete>::new(
            new_content,
            new_punctuation,
            self.new_stamp().as_ref().unwrap().clone(),
            revisable, // * 📌【2024-05-08 11:57:19】就这里是新增的
        );
        let new_task = <Self::Task as TaskConcrete>::from_derive(
            new_sentence,
            new_budget,
            Some(self.current_task().clone()),
            self.current_belief().clone(),
        );
        self.derived_task(new_task);
    }

    /// 模拟`Memory.singlePremiseTask`
    /// * 📝OpenNARS中使用「当前任务」的标点/真值
    ///
    /// # 📄OpenNARS
    ///
    /// Shared final operations by all single-premise rules, called in StructuralRules
    ///
    /// @param newContent The content of the sentence in task
    /// @param newTruth   The truth value of the sentence in task
    /// @param newBudget  The budget value in task
    fn single_premise_task_current(
        &mut self,
        new_content: Term,
        new_truth: Self::Truth,
        new_budget: Self::Budget,
    ) {
        /* 📄OpenNARS源码：
        singlePremiseTask(newContent, currentTask.getSentence().getPunctuation(), newTruth, newBudget); */
        self.single_premise_task(
            new_content,
            self.current_task().sentence().punctuation().clone(),
            new_truth,
            new_budget,
        );
    }

    /// 模拟`Memory.singlePremiseTask`
    /// * 📌支持自定义的「标点」（附带「真值」）
    /// * ⚠️要求`new_stamp`字段非空
    ///
    /// # 📄OpenNARS
    ///
    /// Shared final operations by all single-premise rules, called in StructuralRules
    ///
    ///
    /// @param newContent  The content of the sentence in task
    /// @param punctuation The punctuation of the sentence in task
    /// @param newTruth    The truth value of the sentence in task
    /// @param newBudget   The budget value in task
    fn single_premise_task(
        &mut self,
        new_content: Term,
        punctuation: SentenceType<Self::Truth>,
        new_truth: Self::Truth,
        new_budget: Self::Budget,
    ) {
        /* 📄OpenNARS源码：
        Task parentTask = currentTask.getParentTask();
        if (parentTask != null && newContent.equals(parentTask.getContent())) { // circular structural inference
            return;
        }
        Sentence taskSentence = currentTask.getSentence();
        if (taskSentence.isJudgment() || currentBelief == null) {
            newStamp = new Stamp(taskSentence.getStamp(), getTime());
        } else { // to answer a question with negation in NAL-5 --- move to activated task?
            newStamp = new Stamp(currentBelief.getStamp(), getTime());
        }
        Sentence newSentence = new Sentence(newContent, punctuation, newTruth, newStamp, taskSentence.getRevisable());
        Task newTask = new Task(newSentence, newBudget, currentTask, null);
        derivedTask(newTask); */
        // 判重
        let parent_task = self.current_task().parent_task();
        if let Some(parent_task) = parent_task {
            if *parent_task.content() == new_content {
                return;
            }
        }
        // 产生「新标点」与「新真值」
        let mut new_punctuation = self.current_task().sentence().punctuation().clone();
        // * 🆕🚩【2024-05-08 11:52:03】需要以此将「真值」插入「语句类型/标点」中（「问题」可能没有真值）
        if let SentenceType::Judgement(truth) = &mut new_punctuation {
            *truth = new_truth;
        }
        // 产生「新时间戳」
        let task_sentence = self.current_task().sentence();
        // * 🆕🚩【2024-05-08 14:40:12】此处通过「先决定『旧时间戳』再构造」避免了重复代码与非必要`unwrap`
        let old_stamp = match (task_sentence.is_judgement(), self.current_belief()) {
            (true, _) | (_, None) => task_sentence.stamp(), // * 📄对应`taskSentence.isJudgment() || currentBelief == null`
            (_, Some(belief)) => belief.stamp(),
        };
        let new_stamp = <Self::Stamp as StampConcrete>::with_old(old_stamp, self.time());
        // 语句、任务
        let new_sentence = <Self::Sentence as SentenceConcrete>::new(
            new_content,
            punctuation,
            self.new_stamp().as_ref().unwrap().clone(),
            task_sentence.revisable(), // * 📌【2024-05-08 11:57:19】就这里是新增的
        );
        *self.new_stamp_mut() = Some(new_stamp); // ! 🚩【2024-05-08 15:36:57】必须放在后边：借用检查不通过
        let new_task = <Self::Task as TaskConcrete>::from_derive(
            new_sentence,
            new_budget,
            Some(self.current_task().clone()),
            None,
        );
        self.derived_task(new_task);
    }
}

/// 自动实现，以便添加方法
impl<T: DerivationContext> MemoryDerivationProcess for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
