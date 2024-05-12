//! 🆕有关「推导上下文」与「记忆区」的互操作
//! * 🎯分开存放[「记忆区」](crate::storage::Memory)中与「推导上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4

use super::{DerivationContext, RuleTables};
use crate::{
    _to_string::ToDisplayAndBrief, entity::*, language::Term, nars::DEFAULT_PARAMETERS, storage::*,
};
use narsese::api::NarseseValue;
use navm::output::Output;

/// 记忆区处理：整理与「记忆区」有关的操作
/// * 🚩目前以「记忆区」为中心，以便从「记忆区」处添加方法
/// * 🚩【2024-05-12 15:00:59】因为`RuleTables::transform_task(self);`，要求[`Sized`]
pub trait MemoryProcess: DerivationContext + Sized {
    /// 模拟`Memory.activatedTask`
    /// * 🚩【2024-05-08 11:19:18】因传参需要，部分地方使用[`RC`]
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
            self.new_stamp().clone(),
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
            self.new_stamp().clone(),
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
            self.new_stamp().clone(),
            task_sentence.revisable(), // * 📌【2024-05-08 11:57:19】就这里是新增的
        );
        *self.new_stamp_mut() = new_stamp; // ! 🚩【2024-05-08 15:36:57】必须放在后边：借用检查不通过
        let new_task = <Self::Task as TaskConcrete>::from_derive(
            new_sentence,
            new_budget,
            Some(self.current_task().clone()),
            None,
        );
        self.derived_task(new_task);
    }

    /* ---------- system working workCycle ---------- */

    /// 模拟`Memory.workCycle`
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic working cycle of the system: process new Tasks, then fire a concept
    ///
    /// Called from Reasoner.tick only
    ///
    /// @param clock The current time to be displayed
    fn work_cycle(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        recorder.append(" --- " + clock + " ---\n");
        processNewTask();
        if (noResult()) { // necessary?
            processNovelTask();
        }
        if (noResult()) { // necessary?
            processConcept();
        }
        novelTasks.refresh(); */
        let time = self.time(); // ! 🚩【2024-05-08 15:38:00】必须先获取：借用问题
        self.report(Output::COMMENT {
            content: format!("--- Cycle {time} ---"),
        });
        self.__process_new_task(memory);
        // TODO: `necessary?`可能也是自己需要考虑的问题：是否只在「处理无果」时继续
        if self.no_result() {
            // * 🚩🆕【2024-05-08 14:49:27】合并条件
            self.__process_novel_task(memory);
            self.__process_concept(memory);
        }
        // self.__novel_tasks().refresh(); // ! ❌【2024-05-08 14:49:48】这个方法是「观察者」用的，此处不用
    }

    /// 模拟`Memory.processNewTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Process the newTasks accumulated in the previous workCycle, accept input
    /// ones and those that corresponding to existing concepts, plus one from the
    /// buffer.
    fn __process_new_task(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        Task task;
        int counter = newTasks.size(); // don't include new tasks produced in the current workCycle
        while (counter-- > 0) {
            task = newTasks.removeFirst();
            if (task.isInput() || (termToConcept(task.getContent()) != null)) { // new input or existing concept
                immediateProcess(task);
            } else {
                Sentence s = task.getSentence();
                if (s.isJudgment()) {
                    double d = s.getTruth().getExpectation();
                    if (d > Parameters.DEFAULT_CREATION_EXPECTATION) {
                        novelTasks.putIn(task); // new concept formation
                    } else {
                        recorder.append("!!! Neglected: " + task + "\n");
                    }
                }
            }
        } */
        // let mut task;
        // // * 🚩逆序遍历，实际上又是做了个`-->`语法
        // for counter in (0..self.__new_tasks().len()).rev() {
        //     task = self.__new_tasks_mut().pop_front();
        // }
        // ! ❌【2024-05-08 14:55:26】莫只是照抄OpenNARS的逻辑：此处只是要「倒序取出」而已
        while let Some(task) = memory.__new_tasks_mut().pop_front() {
            let task_concent = task.content();
            if task.is_input() || memory.term_to_concept(task_concent).is_some() {
                self.__immediate_process(task, memory);
            } else {
                let sentence = task.sentence();
                if let SentenceType::Judgement(truth) = sentence.punctuation() {
                    let d = truth.expectation();
                    if d > DEFAULT_PARAMETERS.default_creation_expectation {
                        memory.__novel_tasks_mut().put_in(task);
                    } else {
                        self.report(Output::COMMENT {
                            content: format!("!!! Neglected: {}", task.to_display_long()),
                        });
                    }
                }
            }
        }
    }

    /// 模拟`Memory.processNovelTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Select a novel task to process.
    fn __process_novel_task(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        Task task = novelTasks.takeOut(); // select a task from novelTasks
        if (task != null) {
            immediateProcess(task);
        } */
        let task = memory.__novel_tasks_mut().take_out();
        if let Some(task) = task {
            self.__immediate_process(task, memory);
        }
    }

    /// 模拟`Memory.processConcept`
    ///
    /// # 📄OpenNARS
    ///
    /// Select a concept to fire.
    fn __process_concept(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        currentConcept = concepts.takeOut();
        if (currentConcept != null) {
            currentTerm = currentConcept.getTerm();
            recorder.append(" * Selected Concept: " + currentTerm + "\n");
            concepts.putBack(currentConcept); // current Concept remains in the bag all the time
            currentConcept.fire(); // a working workCycle
        } */
        let concept = memory.__concepts_mut().take_out();
        if let Some(current_concept) = concept {
            let current_term = current_concept.term();
            self.report(Output::COMMENT {
                // * 🚩【2024-05-07 23:05:14】目前仍是将词项转换为「词法Narsese」
                content: format!("* Selected Concept: {}", current_term),
            });
            let key = current_concept.key().clone(); // * 🚩🆕【2024-05-08 15:08:22】拷贝「元素id」以便在「放回」之后仍然能索引
            memory.__concepts_mut().put_back(current_concept);
            // current_concept.fire(); // ! ❌【2024-05-08 15:09:04】不采用：放回了还用，将导致引用混乱
            self.__fire_concept(&key, memory);
        }
    }

    /// 🆕模拟`Concept.fire`
    /// * 📌【2024-05-08 15:06:09】不能让「概念」干「记忆区」干的事
    /// * 📝OpenNARS中从「记忆区」的[「处理概念」](Memory::process_concept)方法中调用
    /// * ⚠️依赖：[`crate::inference::RuleTables`]
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic step in a concept, only called in {@link Memory#processConcept}
    fn __fire_concept(&mut self, concept_key: &Self::Key, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        TaskLink currentTaskLink = taskLinks.takeOut();
        if (currentTaskLink == null) {
            return;
        }
        memory.currentTaskLink = currentTaskLink;
        memory.currentBeliefLink = null;
        memory.getRecorder().append(" * Selected TaskLink: " + currentTaskLink + "\n");
        Task task = currentTaskLink.getTargetTask();
        memory.currentTask = task; // one of the two places where this variable is set
        // memory.getRecorder().append(" * Selected Task: " + task + "\n"); // for
        // debugging
        if (currentTaskLink.getType() == TermLink.TRANSFORM) {
            memory.currentBelief = null;
            RuleTables.transformTask(currentTaskLink, memory); // to turn this into structural inference as below?
        } else {
            int termLinkCount = Parameters.MAX_REASONED_TERM_LINK;
            // while (memory.noResult() && (termLinkCount > 0)) {
            while (termLinkCount > 0) {
                TermLink termLink = termLinks.takeOut(currentTaskLink, memory.getTime());
                if (termLink != null) {
                    memory.getRecorder().append(" * Selected TermLink: " + termLink + "\n");
                    memory.currentBeliefLink = termLink;
                    RuleTables.reason(currentTaskLink, termLink, memory);
                    termLinks.putBack(termLink);
                    termLinkCount--;
                } else {
                    termLinkCount = 0;
                }
            }
        }
        taskLinks.putBack(currentTaskLink); */
        let this = memory
            .__concepts_mut()
            .get_mut(concept_key)
            .expect("不可能失败");
        let current_task_link = this.__task_links_mut().take_out();
        if let Some(current_task_link) = current_task_link {
            // ! 🚩【2024-05-08 16:19:31】必须在「修改」之前先报告（读取）
            self.report(Output::COMMENT {
                content: format!(
                    "* Selected TaskLink: {}",
                    current_task_link.target().to_display_long()
                ),
            });
            *self.current_task_link_mut() = current_task_link;
            *self.current_belief_link_mut() = None; // ? 【2024-05-08 15:41:21】这个有意义吗
            let current_task_link = self.current_task_link();
            let task = current_task_link.target();
            *self.current_task_mut() = task.clone(); // ! 🚩【2024-05-08 16:21:32】目前为「引用计数」需要，暂时如此引入（后续需要解决…）

            // ! 🚩【2024-05-08 16:21:32】↓再次获取，避免借用问题
            if let TermLinkRef::Transform(..) = self.current_task_link().type_ref() {
                *self.current_belief_mut() = None;
                // let current_task_link = self.current_task_link();
                RuleTables::transform_task(self);
            } else {
                let this = memory
                    .__concepts_mut()
                    .get_mut(concept_key)
                    .expect("不可能失败"); // ! 重新获取，以解决借用问题
                                           // * 🚩🆕【2024-05-08 16:52:41】新逻辑：先收集，再处理——避免重复借用
                let mut term_links_to_process = vec![];
                // * 🆕🚩【2024-05-08 16:55:53】简化：实际上只是「最多尝试指定次数下，到了就不尝试」
                for _ in 0..DEFAULT_PARAMETERS.max_reasoned_term_link {
                    let term_link = this.__term_links_mut().take_out();
                    match term_link {
                        Some(term_link) => term_links_to_process.push(term_link),
                        None => break,
                    }
                }
                for term_link in term_links_to_process {
                    self.report(Output::COMMENT {
                        content: format!(
                            "* Selected TermLink: {}",
                            term_link.target().to_display_long()
                        ),
                    });
                    *self.current_belief_link_mut() = Some(term_link);
                    RuleTables::reason(self);
                }
            }
        }
    }

    /* ---------- task processing ---------- */

    /// 模拟`Memory.immediateProcess`
    /// * 📝OpenNARS中对「任务处理」都需要在「常数时间」中运行完毕
    ///   * 💡【2024-05-08 15:34:49】这也是为何「可交换词项变量匹配」需要伪随机「shuffle」
    ///
    /// # 📄OpenNARS
    ///
    /// Immediate processing of a new task,
    /// in constant time Local processing,
    /// in one concept only
    ///
    /// @param task the task to be accepted
    fn __immediate_process(&mut self, task: Self::Task, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        currentTask = task; // one of the two places where this variable is set
        recorder.append("!!! Insert: " + task + "\n");
        currentTerm = task.getContent();
        currentConcept = getConcept(currentTerm);
        if (currentConcept != null) {
            activateConcept(currentConcept, task.getBudget());
            currentConcept.directProcess(task);
        } */
        self.report(Output::COMMENT {
            content: format!("!!! Insert: {}", task.to_display_long()),
        });
        *self.current_task_mut() = task;
        // ! 🚩【2024-05-08 16:07:06】此处不得不使用大量`clone`以解决借用问题；后续可能是性能瓶颈
        let task = /* &** */self.current_task();
        let current_term = task.content().clone();
        let budget = task.budget().clone();
        if let Some(current_concept) = memory.get_concept_or_create(&current_term) {
            let key = current_concept.____key_cloned(); // ! 此处亦需复制，以免借用问题
            memory.activate_concept(&key, &budget);
        }
    }
}

/// 自动实现，以便添加方法
impl<T: DerivationContext> MemoryProcess for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
