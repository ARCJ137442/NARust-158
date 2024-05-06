//! 🎯复刻OpenNARS `nars.entity.Concept`
//! TODO: 着手开始复刻

use super::{
    BudgetValue, Item, Sentence, SentenceConcrete, StampConcrete, Task, TaskConcrete,
    TaskLinkConcrete, TermLinkConcrete, TruthValueConcrete,
};
use crate::{
    entity::{SentenceType, ShortFloat},
    global::Float,
    language::Term,
    storage::{Bag, Memory, TaskLinkBag, TermLinkBag},
};
/// 模拟OpenNARS `nars.entity.Concept`
/// * 🚩【2024-05-04 17:28:30】「概念」首先能被作为「Item」使用
pub trait Concept: Item {
    /// 绑定的「时间戳」类型
    /// * 📌必须是「具体」类型
    type Stamp: StampConcrete;

    /// 绑定的「真值」类型
    /// * 📌必须是「具体」类型
    type Truth: TruthValueConcrete;

    // * ✅至于「元素id」与「预算值」，已在Item约束中绑定

    // * 🚩【2024-05-06 11:23:27】从「语句」到「任务」再到「任务链」，逐个实现关联类型

    /// 绑定的「语句」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    /// * ⚠️【2024-05-06 21:19:01】必须是「具体特征」，不然无法使用「复制」「判等」等方法
    ///   * 💭实际上「复制」是否就意味着「信息就那些」？或许可以考虑移回「抽象特征」？
    ///   TODO: 【2024-05-06 21:20:15】留给以后考量
    type Sentence: SentenceConcrete<Truth = Self::Truth, Stamp = Self::Stamp>;

    /// 绑定的「任务」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    /// * ⚠️【2024-05-06 21:19:01】必须是「具体特征」，不然无法使用「复制」「判等」等方法
    ///   * 💭实际上「复制」是否就意味着「信息就那些」？或许可以考虑移回「抽象特征」？
    ///   TODO: 【2024-05-06 21:20:15】留给以后考量
    type Task: TaskConcrete<Sentence = Self::Sentence, Key = Self::Key, Budget = Self::Budget>;

    /// 绑定的「词项链」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TermLink: TermLinkConcrete<Key = Self::Key, Budget = Self::Budget>;

    /// 绑定的「任务链」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type TaskLink: TaskLinkConcrete<Key = Self::Key, Budget = Self::Budget>;

    /// 模拟`Concept.term`、`Concept.getTerm`
    /// * 🚩只读：OpenNARS仅在构造函数中赋值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `term`
    ///
    /// The term is the unique ID of the concept
    ///
    /// ## `getTerm`
    ///
    /// Return the associated term, called from Memory only
    ///
    /// @return The associated term
    fn term(&self) -> &Term;

    /// 模拟`Concept.taskLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Task links for indirect processing
    fn __task_links(&self) -> &impl TaskLinkBag<Link = Self::TaskLink>;
    /// [`Concept::__task_links`]的可变版本
    fn __task_links_mut(&mut self) -> &mut impl TaskLinkBag<Link = Self::TaskLink>;

    /// 模拟`Concept.termLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Term links between the term and its components and compounds
    fn __term_links(&self) -> &impl TermLinkBag<Link = Self::TermLink>;
    /// [`Concept::__term_links`]的可变版本
    fn __term_links_mut(&mut self) -> &mut impl TermLinkBag<Link = Self::TermLink>;

    /// 模拟`Concept.termLinkTemplates`、`Concept.getTermLinkTemplates`
    /// * 🚩只读：仅在构造函数中被赋值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `termLinkTemplates`
    ///
    /// Link templates of TermLink, only in concepts with CompoundTerm
    ///
    /// ## `getTermLinkTemplates`
    ///
    /// Return the templates for TermLinks, only called in
    /// Memory.continuedProcess
    ///
    /// @return The template get
    fn term_link_templates(&self) -> &[Self::TermLink];

    /// 模拟`Concept.questions`
    /// * 🚩内部读写：仅在内部被使用
    ///
    /// TODO: 考虑作为「共享引用」
    ///
    /// # 📄OpenNARS
    ///
    /// Question directly asked about the term
    fn __questions(&self) -> &[Self::Task];
    /// [`Concept::questions`]的可变版本
    /// * 🚩【2024-05-06 11:49:15】目前使用[`Vec`]：追加、插入、移除
    fn __questions_mut(&mut self) -> &mut Vec<Self::Task>;

    /// 模拟`Concept.questions`
    /// * 🚩内部读写：仅在内部被使用
    ///
    /// TODO: 考虑作为「共享引用」
    ///
    /// # 📄OpenNARS
    ///
    /// Sentences directly made about the term, with non-future tense
    fn __beliefs(&self) -> &[Self::Sentence];
    /// [`Concept::beliefs`]的可变版本
    /// * 🚩【2024-05-06 11:49:15】目前使用[`Vec`]：追加、插入、移除
    fn __beliefs_mut(&mut self) -> &mut Vec<Self::Sentence>;

    // ! ❌【2024-05-06 11:37:01】不实现`Concept.memory`（仅用于内部「袋」的容量获取）
    // ! ❌【2024-05-06 11:37:01】不实现`Concept.entityObserver`

    /* ---------- direct processing of tasks ---------- */

    /// 模拟`Concept.directProcess`
    ///
    /// # 📄OpenNARS
    ///
    /// Directly process a new task. Called exactly once on each task. Using
    /// local information and finishing in a constant time. Provide feedback in
    /// the taskBudget value of the task.
    ///
    /// called in Memory.immediateProcess only
    ///
    /// @param task The task to be processed
    fn direct_process(&mut self, task: &mut Self::Task) {
        /* 📄OpenNARS源码：
        if (task.getSentence().isJudgment()) {
            processJudgment(task);
        } else {
            processQuestion(task);
        }
        if (task.getBudget().aboveThreshold()) { // still need to be processed
            linkToTask(task);
        }
        entityObserver.refresh(displayContent()); */
        use SentenceType::*;
        // * 🚩分派处理
        match task.punctuation() {
            Judgement(..) => self.__process_judgment(task),
            Question => self.__process_question(task),
        }
        // ! 不实现`entityObserver.refresh`
    }

    /// 模拟`Concept.processJudgment`
    ///
    /// # 📄OpenNARS
    ///
    /// To accept a new judgment as isBelief, and check for revisions and
    /// solutions
    ///
    /// @param task The judgment to be accepted
    /// @param task The task to be processed
    /// @return Whether to continue the processing of the task
    fn __process_judgment(&mut self, task: &mut Self::Task) {
        /* 📄OpenNARS源码：
        Sentence judgment = task.getSentence();
        Sentence oldBelief = evaluation(judgment, beliefs);
        if (oldBelief != null) {
            Stamp newStamp = judgment.getStamp();
            Stamp oldStamp = oldBelief.getStamp();
            if (newStamp.equals(oldStamp)) {
                if (task.getParentTask().getSentence().isJudgment()) {
                    task.getBudget().decPriority(0); // duplicated task
                } // else: activated belief
                return;
            } else if (LocalRules.revisable(judgment, oldBelief)) {
                memory.newStamp = Stamp.make(newStamp, oldStamp, memory.getTime());
                if (memory.newStamp != null) {
                    memory.currentBelief = oldBelief;
                    LocalRules.revision(judgment, oldBelief, false, memory);
                }
            }
        }
        if (task.getBudget().aboveThreshold()) {
            for (Task ques : questions) {
                // LocalRules.trySolution(ques.getSentence(), judgment, ques, memory);
                LocalRules.trySolution(judgment, ques, memory);
            }
            addToTable(judgment, beliefs, Parameters.MAXIMUM_BELIEF_LENGTH);
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.processQuestion`
    /// * 📝OpenNARS原先返回的是「回答真值的期望」
    ///   * 🚩【2024-05-06 11:59:00】实际上并没有用，故不再返回
    /// * 📝OpenNARS仅在「直接处理」时用到它
    ///   * 🚩【2024-05-06 11:59:54】实际上直接变为私有方法，也不会妨碍到具体运行
    ///
    /// # 📄OpenNARS
    ///
    /// To answer a question by existing beliefs
    ///
    /// @param task The task to be processed
    /// @return Whether to continue the processing of the task
    fn __process_question(&mut self, task: &mut Self::Task) /* -> <Self::Truth as TruthValue>::E */
    {
        /* 📄OpenNARS源码：
        Sentence ques = task.getSentence();
        boolean newQuestion = true;
        if (questions != null) {
            for (Task t : questions) {
                Sentence q = t.getSentence();
                if (q.getContent().equals(ques.getContent())) {
                    ques = q;
                    newQuestion = false;
                    break;
                }
            }
        }
        if (newQuestion) {
            questions.add(task);
        }
        if (questions.size() > Parameters.MAXIMUM_QUESTIONS_LENGTH) {
            questions.remove(0); // FIFO
        }
        Sentence newAnswer = evaluation(ques, beliefs);
        if (newAnswer != null) {
            // LocalRules.trySolution(ques, newAnswer, task, memory);
            LocalRules.trySolution(newAnswer, task, memory);
            return newAnswer.getTruth().getExpectation();
        } else {
            return 0.5f;
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.linkToTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Link to a new task from all relevant concepts for continued processing in
    /// the near future for unspecified time.
    ///
    /// The only method that calls the TaskLink constructor.
    ///
    /// @param task    The task to be linked
    /// @param cont
    fn __link_to_task(&mut self, task: &mut Self::Task) {
        /* 📄OpenNARS源码：
        BudgetValue taskBudget = task.getBudget();
        TaskLink taskLink = new TaskLink(task, null, taskBudget); // link type: SELF
        insertTaskLink(taskLink);
        if (term instanceof CompoundTerm) {
            if (termLinkTemplates.size() > 0) {
                BudgetValue subBudget = BudgetFunctions.distributeAmongLinks(taskBudget, termLinkTemplates.size());
                if (subBudget.aboveThreshold()) {
                    Term componentTerm;
                    Concept componentConcept;
                    for (TermLink termLink : termLinkTemplates) {
                        // if (!(task.isStructural() && (termLink.getType() == TermLink.TRANSFORM))) {
                        // // avoid circular transform
                        taskLink = new TaskLink(task, termLink, subBudget);
                        componentTerm = termLink.getTarget();
                        componentConcept = memory.getConcept(componentTerm);
                        if (componentConcept != null) {
                            componentConcept.insertTaskLink(taskLink);
                        }
                        // }
                    }
                    buildTermLinks(taskBudget); // recursively insert TermLink
                }
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.addToTable`
    /// * 🚩实际上是个静态方法：不依赖实例
    /// * 🚩对「物品列表」使用标准库的[`Vec`]类型，与[`Concept::__beliefs_mut`]同步
    ///
    /// # 📄OpenNARS
    ///
    /// Add a new belief (or goal) into the table Sort the beliefs/goals by rank,
    /// and remove redundant or low rank one
    ///
    /// @param newSentence The judgment to be processed
    /// @param table       The table to be revised
    /// @param capacity    The capacity of the table
    fn __add_to_table(sentence: &Self::Sentence, table: &mut Vec<Self::Sentence>, capacity: usize) {
        /* 📄OpenNARS源码：
        float rank1 = BudgetFunctions.rankBelief(newSentence); // for the new isBelief
        Sentence judgment2;
        float rank2;
        int i;
        for (i = 0; i < table.size(); i++) {
            judgment2 = table.get(i);
            rank2 = BudgetFunctions.rankBelief(judgment2);
            if (rank1 >= rank2) {
                if (newSentence.equivalentTo(judgment2)) {
                    return;
                }
                table.add(i, newSentence);
                break;
            }
        }
        if (table.size() >= capacity) {
            while (table.size() > capacity) {
                table.remove(table.size() - 1);
            }
        } else if (i == table.size()) {
            table.add(newSentence);
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.evaluation`
    /// * 📝实际上不依赖实例，是个静态方法
    ///
    /// # 📄OpenNARS
    ///
    /// Evaluate a query against beliefs (and desires in the future)
    ///
    /// @param query The question to be processed
    /// @param list  The list of beliefs to be used
    /// @return The best candidate belief selected
    fn __evaluation(query: Self::Sentence, list: &[Self::Sentence]) -> Option<&Self::Sentence> {
        /* 📄OpenNARS源码：
        if (list == null) {
            return null;
        }
        float currentBest = 0;
        float beliefQuality;
        Sentence candidate = null;
        for (Sentence judgment : list) {
            beliefQuality = LocalRules.solutionQuality(query, judgment);
            if (beliefQuality > currentBest) {
                currentBest = beliefQuality;
                candidate = judgment;
            }
        }
        return candidate; */
        todo!("// TODO: 有待实现")
    }

    /* ---------- insert Links for indirect processing ---------- */

    /// 模拟`Concept.insertTaskLink`
    ///
    /// # 📄OpenNARS
    ///
    /// Insert a TaskLink into the TaskLink bag
    ///
    /// called only from Memory.continuedProcess
    ///
    /// @param taskLink The termLink to be inserted
    fn insert_task_link(&mut self, task_link: Self::TaskLink) {
        /* 📄OpenNARS源码：
        BudgetValue taskBudget = taskLink.getBudget();
        taskLinks.putIn(taskLink);
        memory.activateConcept(this, taskBudget); */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.buildTermLinks`
    ///
    /// # 📄OpenNARS
    ///
    /// Recursively build TermLinks between a compound and its components
    ///
    /// called only from Memory.continuedProcess
    ///
    /// @param taskBudget The BudgetValue of the task
    fn build_term_links(&mut self, task_budget: &Self::Budget) {
        /* 📄OpenNARS源码：
        Term t;
        Concept concept;
        TermLink termLink1, termLink2;
        if (termLinkTemplates.size() > 0) {
            BudgetValue subBudget = BudgetFunctions.distributeAmongLinks(taskBudget, termLinkTemplates.size());
            if (subBudget.aboveThreshold()) {
                for (TermLink template : termLinkTemplates) {
                    if (template.getType() != TermLink.TRANSFORM) {
                        t = template.getTarget();
                        concept = memory.getConcept(t);
                        if (concept != null) {
                            termLink1 = new TermLink(t, template, subBudget);
                            insertTermLink(termLink1); // this termLink to that
                            termLink2 = new TermLink(term, template, subBudget);
                            concept.insertTermLink(termLink2); // that termLink to this
                            if (t instanceof CompoundTerm) {
                                concept.buildTermLinks(subBudget);
                            }
                        }
                    }
                }
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`Concept.insertTermLink`
    ///
    /// # 📄OpenNARS
    ///
    /// Insert a TermLink into the TermLink bag
    ///
    /// called from buildTermLinks only
    ///
    /// @param termLink The termLink to be inserted
    fn insert_term_link(&mut self, term_link: Self::TermLink) {
        /* 📄OpenNARS源码：
        termLinks.putIn(termLink); */
        self.__term_links_mut().put_in(term_link);
    }

    /* ---------- access local information ---------- */

    // ! ❌【2024-05-06 18:45:48】暂不模拟`toString`与`toStringLong`、`toStringIfNotNull`
    // ? ℹ️似乎`toString`还要用到`NARSBatch.isStandAlone()`这种「全局属性」

    /// 模拟`Concept.________`
    /// * ⚠️覆盖原先对[`BudgetValue::quality`]的实现
    ///   * ❓Rust似乎不太能迁移这类「覆盖」的情形——只能靠「重名歧义」提醒
    ///     * 🚩不过后续可以通过「完全限定语法」指定`<self as Concept>::quality`来调用，并且也能提醒在所用之处实现
    ///   * ✅在「概念袋」中的访问，仍然使用其作为[`Item`]的原始实现（[内部「预算值」](Self::Budget)的[质量](BudgetValue::quality)）
    ///     * ℹ️【2024-05-06 19:01:45】已通过OpenNARS调试得到验证：「概念」有两种预算值
    ///       * 第一种是其作为「Item」访问内部[「预算值」](Item::Budget)所得到的「质量」
    ///       * 第二种即为此处「概念」作为一个「整体」所得到的「质量」
    ///     * 📌【2024-05-06 19:01:37】目前认为此处实际上无需出现「方法覆盖」，因为这种覆盖本身就是无效的
    ///       * 第一种走的是`self.budget.quality()`而非`self.quality()`（在实际推理传参时）
    ///       * ✅【2024-05-06 19:22:27】在OpenNARS 3.0.4中，经过断点调试验证，此处亦同奏效
    /// * 📝OpenNARS只会在「预算函数」的[「激活」](crate::inference::BudgetFunctions::activate)处调用
    ///   * 📝同时这个「激活」函数，只会被[「记忆区」](crate::storage::Memory)的[「激活概念」](crate::storage::Memory::activate_concept)调用
    ///   * 📄OpenNARS 3.0.4中亦是「使用场合单一」
    /// * 🚩【2024-05-06 18:54:21】目前的决策：重命名为`total_quality`，以便和「其作为[`Item`]时的『质量』」相区分
    ///
    /// # 📄OpenNARS
    ///
    /// Recalculate the quality of the concept [to be refined to show
    /// extension/intension balance]
    ///
    /// @return The quality value
    fn total_quality(&self) -> <Self::Budget as BudgetValue>::E {
        /* 📄OpenNARS源码：
        float linkPriority = termLinks.averagePriority();
        float termComplexityFactor = 1.0f / term.getComplexity();
        return UtilityFunctions.or(linkPriority, termComplexityFactor); */
        let from = <<Self::Budget as BudgetValue>::E as ShortFloat>::from_float;
        let link_priority = from(self.__term_links().average_priority());
        let term_complexity_factor = from(1.0 / self.term().complexity() as Float);
        link_priority | term_complexity_factor
    }

    /// 模拟`Concept.getBelief`
    /// * 🚩目前「记忆区」在参数调用中引入
    ///
    /// # 📄OpenNARS
    ///
    /// Select a isBelief to interact with the given task in inference
    ///
    /// get the first qualified one
    ///
    /// only called in RuleTables.reason
    ///
    /// @param task The selected task
    /// @return The selected isBelief
    fn get_belief(
        &self,
        memory: &impl Memory<Concept = Self>,
        task: &Self::Task,
    ) -> Option<Self::Sentence> {
        /* 📄OpenNARS源码：
        Sentence taskSentence = task.getSentence();
        for (Sentence belief : beliefs) {
            memory.getRecorder().append(" * Selected Belief: " + belief + "\n");
            memory.newStamp = Stamp.make(taskSentence.getStamp(), belief.getStamp(), memory.getTime());
            if (memory.newStamp != null) {
                Sentence belief2 = (Sentence) belief.clone(); // will this mess up priority adjustment?
                return belief2;
            }
        }
        return null; */
        let task_sentence = task.sentence();
        for belief in self.__beliefs() {
            let new_stamp =
                Self::Stamp::from_merge(task_sentence.stamp(), belief.stamp(), memory.time());
            if new_stamp.is_some() {
                // ? 实际上又不要这个时间戳，实际上就是要了个「判断是否重复」的逻辑
                let belief2 = belief.clone();
                return Some(belief2);
            }
        }
        None
    }

    /* ---------- main loop ---------- */

    /// 模拟`Concept.fire`
    /// * 📝OpenNARS中从「记忆区」的[「处理概念」](Memory::process_concept)方法中调用
    /// * ⚠️依赖：[`crate::inference::RuleTables`]
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic step in a concept, only called in {@link Memory#processConcept}
    fn fire() {
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
        todo!("// TODO: 有待实现")
    }

    // ! ❌【2024-05-06 21:23:00】暂不实现与「呈现」「观察」有关的方法
}

/// 「概念」的具体类型
pub trait ConceptConcrete: Concept + Sized {
    /* ---------- constructor and initialization ---------- */

    /// 模拟`new Concept(Term tm, Memory memory)`
    /// * 🚩具体的「创建空数组」「创建空袋」交由「初代实现」实现
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor, called in Memory.getConcept only
    ///
    /// @param tm     A term corresponding to the concept
    /// @param memory A reference to the memory
    fn new(term: Term) -> Self;
}

/// 初代实现
mod impl_v1 {
    use super::*;

    /// TODO: 初代实现
    pub struct ConceptV1 {
        // TODO: 添加字段
    }

    // TODO: 有待迁移到`ConceptConcrete`实现
    impl ConceptV1 {
        fn new(term: Term) -> Self {
            /* 📄OpenNARS源码：
            super(tm.getName());
            term = tm;
            this.memory = memory;
            questions = new ArrayList<>();
            beliefs = new ArrayList<>();
            taskLinks = new TaskLinkBag(memory);
            termLinks = new TermLinkBag(memory);
            if (tm instanceof CompoundTerm) {
                termLinkTemplates = ((CompoundTerm) tm).prepareComponentLinks();
            } */
            // TODO: 复刻逻辑
            Self {}
        }
    }
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
