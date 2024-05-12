//! 🆕有关「推导上下文」与「概念」的互操作
//! * 🎯分开存放[「概念」](crate::entity::Concept)中与「推导上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4

use super::DerivationContext;
use crate::{entity::*, global::Float, inference::*, language::Term, storage::*};

///
/// * 🚩因为`<Self as LocalRules>::solution_quality`要求[`Sized`]
pub trait ConceptProcess: DerivationContext + Sized {
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
        let judgement = task.sentence();
        // let old_belief = Self::__evaluation(judgement, self.__beliefs());
        // TODO: ❓【2024-05-08 17:43:59】有待解决「需要额外引入的『推理上下文』」问题
        //   * 💭可能需要把这一系列「process」迁移出去，如`trait ConceptProcess: ReasonContext`
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
    fn __evaluation<'l, Context>(
        query: &Self::Sentence,
        list: &'l [Self::Sentence],
    ) -> Option<&'l Self::Sentence>
    where
        Context: ReasonContext<Concept = Self, Sentence = Self::Sentence>,
    {
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
        let current_best: Float = 0.0;
        for judgement in list {
            let belief_quality = <Self as LocalRules>::solution_quality(Some(query), judgement);
        }
        todo!("// TODO: 有待实现")
    }

    /* ---------- insert Links for indirect processing ---------- */

    /// 模拟`Concept.insertTaskLink`
    /// * 🚩【2024-05-07 22:29:32】应该是个关联函数
    ///   * 💭插入「词项链」要使用「记忆区」但「记忆区」却又循环操作「概念」本身（获取所有权），这不会冲突吗？
    ///
    /// TODO: 🏗️【2024-05-07 22:31:05】有待适配
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
    fn insert_term_link(&mut self, term_link: Self::TermLink, concept: &mut Self::Concept) {
        /* 📄OpenNARS源码：
        termLinks.putIn(termLink); */
        concept.__term_links_mut().put_in(term_link);
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
    fn get_belief(&self, concept: &Self::Concept, task: &Self::Task) -> Option<Self::Sentence> {
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
        for belief in concept.__beliefs() {
            let new_stamp =
                Self::Stamp::from_merge(task_sentence.stamp(), belief.stamp(), self.time());
            if new_stamp.is_some() {
                // ? 实际上又不要这个时间戳，实际上就是要了个「判断是否重复」的逻辑
                let belief2 = belief.clone();
                return Some(belief2);
            }
        }
        None
    }
}
