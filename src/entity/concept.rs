//! 🎯复刻OpenNARS `nars.entity.Concept`
//! TODO: 着手开始复刻

use super::{
    Item, Sentence, SentenceConcrete, SentenceType, Stamp, StampConcrete, Task, TaskConcrete,
    TaskLink, TaskLinkConcrete, TermLinkConcrete, TruthValue, TruthValueConcrete,
};
use crate::{
    language::Term,
    storage::{TaskLinkBag, TermLinkBag},
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
    type Sentence: Sentence<Truth = Self::Truth, Stamp = Self::Stamp>;

    /// 绑定的「任务」
    /// * 🎯每个实现中只会实现一种类型，用于统一多个函数的参数
    type Task: Task<Sentence = Self::Sentence, Key = Self::Key, Budget = Self::Budget>;

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
    /// <p>
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

    /// 模拟`Concept.________`
    ///
    /// # 📄OpenNARS
    ///
    fn ________() {
        /* 📄OpenNARS源码： */
        todo!("// TODO: 有待实现")
    }

    // TODO: 其它方法
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
