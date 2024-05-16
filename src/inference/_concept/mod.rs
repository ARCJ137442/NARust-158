//! 🆕有关「推导上下文」与「概念」的互操作
//! * 🎯分开存放[「概念」](crate::entity::Concept)中与「推导上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4

use self::language::Term;
use super::DerivationContext;
use crate::{entity::*, global::Float, inference::*, nars::DEFAULT_PARAMETERS, storage::*, *};
use navm::output::Output;

///
/// * 🚩因为`<Self as LocalRules>::solution_quality`要求[`Sized`]
pub trait ConceptProcess: DerivationContext {
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
    fn direct_process(&mut self, concept: &mut Self::Concept, task: &mut Self::Task) {
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
            // 判断
            Judgement(..) => self.__process_judgment(concept, task),
            // 问题 | 🚩此处无需使用返回值，故直接`drop`掉（并同时保证类型一致）
            // * 📌【2024-05-15 17:08:44】此处因为需要「将新问题添加到『问题列表』中」而使用可变引用
            Question => drop(self.__process_question(concept, task)),
        }
        // ! 不实现`entityObserver.refresh`
    }

    /// 模拟`Concept.processJudgment`
    /// * ⚠️【2024-05-12 17:13:50】此处假定`task`
    ///   * 具有「父任务」即`parent_task`非空
    ///   * 可变：需要改变其预算值
    ///
    /// # 📄OpenNARS
    ///
    /// To accept a new judgment as isBelief, and check for revisions and
    /// solutions
    ///
    /// @param task The judgment to be accepted
    /// @param task The task to be processed
    /// @return Whether to continue the processing of the task
    fn __process_judgment(&mut self, concept: &Self::Concept, task: &mut Self::Task) {
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
        let old_belief = self.__evaluation(judgement, concept.__beliefs());
        if let Some(old_belief) = old_belief {
            let new_stamp = judgement.stamp();
            let old_stamp = old_belief.stamp();
            // 若为「重复任务」——优先级放到最后
            if new_stamp.equals(old_stamp) {
                if task.parent_task().as_ref().unwrap().is_judgement() {
                    task.budget_mut().dec_priority(Self::ShortFloat::ZERO);
                }
                return;
            } else if <Self as LocalRules>::revisable(judgement, old_belief) {
                *self.new_stamp_mut() =
                    <Self::Stamp as StampConcrete>::from_merge(new_stamp, old_stamp, self.time());
                if self.new_stamp().is_some() {
                    // 🆕此处复制了「旧信念」以便设置值
                    // TODO: ❓是否需要这样：有可能后续处在「概念」中的信念被修改了，这里所指向的「信念」却没有
                    *self.current_belief_mut() = Some(old_belief.clone());
                    let old_belief = self.current_belief().as_ref().unwrap();
                    let old_belief = &old_belief.clone();
                    // ! 📌依靠复制，牺牲性能以**解决引用问题**（不然会引用`self`）
                    // * ❓↑但，这样会不会受到影响
                    self.revision(judgement, old_belief, false);
                }
            }
        }
        if task
            .budget()
            .above_threshold(ShortFloat::from_float(DEFAULT_PARAMETERS.budget_threshold))
        {
            for question in concept.__questions() {
                self.try_solution(judgement, question);
            }
        }
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
    fn __process_question(
        &mut self,
        concept: &mut Self::Concept,
        task: &mut Self::Task,
    ) -> Self::ShortFloat {
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
        // * 🚩复刻逻辑 in 借用规则：先寻找答案，再插入问题
        let mut question = task.sentence();
        let mut is_new_question = true;
        // * 🚩找到自身「问题列表」中与「任务」相同的「问题」
        for task in concept.__questions() {
            // TODO: 【2024-05-12 23:42:08】有待进一步实现
            let task_question = task.sentence();
            if question == task_question {
                question = task_question;
                is_new_question = false;
                break;
            }
        }
        // * 🚩先尝试回答
        let result;
        let new_answer = self.__evaluation(question, concept.__beliefs());
        if let Some(new_answer) = new_answer {
            LocalRules::try_solution(self, new_answer, task);
            result = new_answer.truth().unwrap().expectation(); // ! 保证里边都是「判断」
        } else {
            result = 0.5;
        }
        // * 🚩再插入问题
        {
            // * 🚩新问题⇒加入「概念」已有的「问题列表」中（有限大小缓冲区）
            if is_new_question {
                // * ⚠️此处复制了「任务」以解决「所有权分配」问题
                concept.__questions_mut().push(task.clone());
            }
            // * 🚩有限大小缓冲区：若加入后大小溢出，则「先进先出」（在Rust语境下任务被销毁）
            // TODO: 后续要实现一个「固定大小缓冲区队列」？
            if concept.__questions().len() > DEFAULT_PARAMETERS.maximum_questions_length {
                concept.__questions_mut().remove(0);
            }
        }
        // * 🚩最后返回生成的返回值
        Self::ShortFloat::from_float(result)
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
    fn __evaluation<'l>(
        &mut self,
        query: &Self::Sentence,
        list: &'l [Self::Sentence],
    ) -> Option<&'l Self::Sentence> {
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
        /* let mut current_best: Float = 0.0;
        let mut candidate = None;
        for judgement in list {
            let belief_quality =
                <Self as LocalRules>::solution_quality(Some(query), judgement).to_float();
            if belief_quality > current_best {
                current_best = belief_quality;
                candidate = Some(judgement);
            }
        } */
        // ! ⚠️【2024-05-16 00:42:47】使用迭代器的方法有所不同：若有多个相等，则最后一个会被选中（而非最先一个）
        // * ✅【2024-05-16 00:43:35】解决方案：迭代器逆向遍历
        let candidate = list
            .iter()
            .rev() // * 🚩【2024-05-16 00:44:00】逆向遍历以保证「相同质量⇒最先一个」
            .max_by_key(|judgement| <Self as LocalRules>::solution_quality(Some(query), judgement));
        candidate
    }

    /// 模拟`Concept.linkToTask`
    /// * ⚠️【2024-05-15 17:20:47】涉及大量共享引用
    ///   * 💫共享引用策源地：如何在无GC语言中尽可能减少这类共享引用，是个问题
    ///     * ❗特别是在「任务」还分散在各个「概念」的「任务链」中的情况
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
        let task_budget = task.budget();
        // TODO: 词项链/任务链「模板」机制
        // * 💫【2024-05-15 17:38:16】循环引用，频繁修改、结构相异……
        // let task_link = TaskLinkConcrete::new();
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

    /// 模拟`CompoundTerm.prepareComponentLinks`
    /// * 🚩返回一个「准备好的词项链模板列表」
    /// * 📝尚未实装：需要在构造函数中预先加载
    ///
    /// # 📄OpenNARS
    ///
    /// Build TermLink templates to constant components and sub-components
    ///
    /// The compound type determines the link type; the component type determines
    /// whether to build the link.
    ///
    /// @return A list of TermLink templates
    fn prepare_component_link_templates(self_term: &Term) -> Vec<Self::TermLink> {
        /* 📄OpenNARS源码：
        ArrayList<TermLink> componentLinks = new ArrayList<>();
        short type = (self instanceof Statement) ? TermLink.COMPOUND_STATEMENT : TermLink.COMPOUND; // default
        prepareComponentLinks(self, componentLinks, type, self);
        return componentLinks; */
        let mut component_links = vec![];
        // * 🚩【2024-05-15 19:13:40】因为此处与「索引」绑定，故使用默认值当索引
        // * 💫不可能完全照搬了
        let link_type = match self_term.instanceof_statement() {
            true => TermLinkType::CompoundStatement(vec![]),
            false => TermLinkType::Compound(vec![]),
        };
        // * 🚩朝里边添加词项链模板
        Self::__prepare_component_link_templates(
            self_term,
            &mut component_links,
            &link_type,
            self_term,
        );
        component_links
    }

    /// 模拟`CompoundTerm.prepareComponentLinks`
    /// * 📌【2024-05-15 18:07:27】目前考虑直接使用值，而非共享引用
    /// * 📝【2024-05-15 18:05:01】OpenNARS在这方面做得相对复杂
    /// * 💫【2024-05-15 18:05:06】目前尚未理清其中原理
    ///
    /// # 📄OpenNARS
    ///
    /// Collect TermLink templates into a list, go down one level except in
    /// special cases
    ///
    /// @param componentLinks The list of TermLink templates built so far
    /// @param type           The type of TermLink to be built
    /// @param term           The CompoundTerm for which the links are built
    fn __prepare_component_link_templates(
        self_term: &Term,
        component_links: &mut Vec<Self::TermLink>,
        type_: &TermLinkType,
        term: &Term,
    ) -> Vec<Self::TermLink> {
        /* 📄OpenNARS源码：
        for (int i = 0; i < term.size(); i++) { // first level components
            final Term t1 = term.componentAt(i);
            if (t1.isConstant()) {
                componentLinks.add(new TermLink(t1, type, i));
            }
            if (((self instanceof Equivalence) || ((self instanceof Implication) && (i == 0)))
                    && ((t1 instanceof Conjunction) || (t1 instanceof Negation))) {
                prepareComponentLinks(((CompoundTerm) t1), componentLinks, TermLink.COMPOUND_CONDITION,
                        (CompoundTerm) t1);
            } else if (t1 instanceof CompoundTerm) {
                for (int j = 0; j < ((CompoundTerm) t1).size(); j++) { // second level components
                    final Term t2 = ((CompoundTerm) t1).componentAt(j);
                    if (t2.isConstant()) {
                        if ((t1 instanceof Product) || (t1 instanceof ImageExt) || (t1 instanceof ImageInt)) {
                            if (type == TermLink.COMPOUND_CONDITION) {
                                componentLinks.add(new TermLink(t2, TermLink.TRANSFORM, 0, i, j));
                            } else {
                                componentLinks.add(new TermLink(t2, TermLink.TRANSFORM, i, j));
                            }
                        } else {
                            componentLinks.add(new TermLink(t2, type, i, j));
                        }
                    }
                    if ((t2 instanceof Product) || (t2 instanceof ImageExt) || (t2 instanceof ImageInt)) {
                        for (int k = 0; k < ((CompoundTerm) t2).size(); k++) {
                            final Term t3 = ((CompoundTerm) t2).componentAt(k);
                            if (t3.isConstant()) { // third level
                                if (type == TermLink.COMPOUND_CONDITION) {
                                    componentLinks.add(new TermLink(t3, TermLink.TRANSFORM, 0, i, j, k));
                                } else {
                                    componentLinks.add(new TermLink(t3, TermLink.TRANSFORM, i, j, k));
                                }
                            }
                        }
                    }
                }
            }
        } */
        todo!("// TODO: 待实现")
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
                // * 📝实际逻辑即「有共有证据⇒不要推理」
                // ? 实际上又不要这个时间戳，实际上就是要了个「判断是否重复」的逻辑
                let belief2 = belief.clone();
                return Some(belief2);
            }
        }
        None
    }

    /* ---------- main loop ---------- */

    /// 🆕模拟`Concept.fire`
    /// * 📌【2024-05-08 15:06:09】不能让「概念」干「记忆区」干的事
    /// * 📝OpenNARS中从「记忆区」的[「处理概念」](Memory::process_concept)方法中调用
    /// * ⚠️依赖：[`crate::inference::RuleTables`]
    /// * 🚩【2024-05-12 16:08:58】现在独立在「推导上下文」中，
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic step in a concept, only called in {@link Memory#processConcept}
    fn __fire_concept(&mut self, concept: &mut Self::Concept) {
        /* 📄OpenNARS源码：
        TaskLink currentTaskLink = taskLinks.takeOut();
        if (currentTaskLink == null) {
            return;
        }
        memory.currentTaskLink = currentTaskLink;
        memory.currentBeliefLink = null;
        memory.getRecorder().append(" * Selected TaskLink: " + currentTaskLink + "\n");
        Task task = currentTaskLink.getTargetTask();
        memory.currentTask = task; // one of the two places where concept variable is set
        // memory.getRecorder().append(" * Selected Task: " + task + "\n"); // for
        // debugging
        if (currentTaskLink.getType() == TermLink.TRANSFORM) {
            memory.currentBelief = null;
            RuleTables.transformTask(currentTaskLink, memory); // to turn concept into structural inference as below?
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
        let current_task_link = concept.__task_links_mut().take_out();
        if let Some(current_task_link) = current_task_link {
            // ! 🚩【2024-05-08 16:19:31】必须在「修改」之前先报告（读取）
            self.report(Output::COMMENT {
                content: format!(
                    "* Selected TaskLink: {}",
                    current_task_link.target().to_display_long()
                ),
            });
            *self.current_task_link_mut() = Some(current_task_link);
            *self.current_belief_link_mut() = None; // ? 【2024-05-08 15:41:21】这个有意义吗

            // 此处设定上下文状态
            let current_task_link = self.current_task_link().as_ref().unwrap();
            let task = current_task_link.target();
            *self.current_task_mut() = task.clone(); // ! 🚩【2024-05-08 16:21:32】目前为「引用计数」需要，暂时如此引入（后续需要解决…）

            // ! 🚩【2024-05-08 16:21:32】↓再次获取，避免借用问题
            if let TermLinkRef::Transform(..) =
                self.current_task_link().as_ref().unwrap().type_ref()
            {
                *self.current_belief_mut() = None;
                // let current_task_link = self.current_task_link();
                RuleTables::transform_task(self);
            } else {
                // * 🚩🆕【2024-05-08 16:52:41】新逻辑：先收集，再处理——避免重复借用
                let mut term_links_to_process = vec![];
                // * 🆕🚩【2024-05-08 16:55:53】简化：实际上只是「最多尝试指定次数下，到了就不尝试」
                for _ in 0..DEFAULT_PARAMETERS.max_reasoned_term_link {
                    let term_link = concept.__term_links_mut().take_out();
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
                    // * 🔥启动推理
                    RuleTables::reason(self);
                }
            }
        }
    }
}

/// 自动实现，以便添加方法
impl<T: DerivationContext> ConceptProcess for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
