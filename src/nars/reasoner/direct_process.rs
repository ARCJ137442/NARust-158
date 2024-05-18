//! 推理器有关「直接推理/立即推理」的功能
//! * 🎯模拟以`Memory.immediateProcess`为入口的「直接推理」
//! * 🎯将其中有关「直接推理」的代码摘录出来
//!   * 📌处理新任务(内部) from 工作周期(@记忆区)
//!   * 📌处理新近任务(内部) from 工作周期(@记忆区)
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部 @概念) from 直接处理
//!   * 📌处理问题(内部 @概念) from 直接处理
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组

use crate::{entity::*, inference::*, nars::*, storage::*, *};
use navm::output::Output;

/// 推理器与「工作周期」有关的功能
pub trait ReasonerDirectProcess<C: ReasonContext>: Reasoner<C> {
    /// 🆕本地直接推理
    /// * 🚩最终只和「本地规则」[`LocalRules`]有关
    fn direct_process(&mut self, context: &mut Self::DerivationContextDirect) -> bool {
        // * 🚩处理新任务
        self.__process_new_task(context);

        // TODO: `necessary?`可能也是自己需要考虑的问题：是否只在「处理无果」时继续
        if context.no_result() {
            // * 🚩处理新近任务
            self.__process_novel_task(context);
        }

        // * 🚩返回「是否要继续」 | 不与「概念推理」的功能耦合
        !context.no_result()
    }

    /// 模拟`Memory.processNewTask`
    /// * 🚩【2024-05-17 21:25:46】从「记忆区」换成「直接推理上下文」
    ///
    /// # 📄OpenNARS
    ///
    /// Process the newTasks accumulated in the previous workCycle, accept input
    /// ones and those that corresponding to existing concepts, plus one from the
    /// buffer.
    fn __process_new_task(&mut self, context: &mut Self::DerivationContextDirect) {
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
        while let Some(task) = self.__new_tasks_mut().pop_front() {
            let task_concent = task.content();
            if task.is_input() || self.memory().term_to_concept(task_concent).is_some() {
                self.__immediate_process(task, context);
            } else {
                let sentence = task.sentence();
                if let SentenceType::Judgement(truth) = sentence.punctuation() {
                    let d = truth.expectation();
                    if d > DEFAULT_PARAMETERS.default_creation_expectation {
                        self.__novel_tasks_mut().put_in(task);
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
    fn __process_novel_task(&mut self, context: &mut Self::DerivationContextDirect) {
        /* 📄OpenNARS源码：
        Task task = novelTasks.takeOut(); // select a task from novelTasks
        if (task != null) {
            immediateProcess(task);
        } */
        let task = self.__novel_tasks_mut().take_out();
        if let Some(task) = task {
            self.__immediate_process(task, context);
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
    fn __immediate_process(&mut self, task: C::Task, context: &mut Self::DerivationContextDirect) {
        /* 📄OpenNARS源码：
        currentTask = task; // one of the two places where concept variable is set
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
        *context.current_task_mut() = Some(task);
        // * 🚩【2024-05-17 21:28:14】放入后又拿出，以在「置入所有权后获取其引用」
        let task = context.current_task().as_ref().unwrap();
        // ! 🚩【2024-05-08 16:07:06】此处不得不使用大量`clone`以解决借用问题；后续可能是性能瓶颈
        let current_term = task.content().clone();
        let budget = task.budget().clone();
        if let Some(current_concept) = self.memory_mut().get_concept_or_create(&current_term) {
            let key = current_concept.____key_cloned(); // ! 此处亦需复制，以免借用问题
            self.memory_mut().activate_concept(&key, &budget);
        }
    }

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
    fn __direct_process_concept(
        context: &mut Self::DerivationContextDirect,
        concept: &mut C::Concept,
        task: &mut C::Task,
    ) {
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
            Judgement(..) => Self::__process_judgment(context, concept, task),
            // 问题 | 🚩此处无需使用返回值，故直接`drop`掉（并同时保证类型一致）
            // * 📌【2024-05-15 17:08:44】此处因为需要「将新问题添加到『问题列表』中」而使用可变引用
            Question => drop(Self::__process_question(context, concept, task)),
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
    fn __process_judgment(
        context: &mut Self::DerivationContextDirect,
        concept: &C::Concept,
        task: &mut C::Task,
    ) {
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
        let old_belief = Self::__evaluation(judgement, concept.__beliefs());
        if let Some(old_belief) = old_belief {
            let new_stamp = judgement.stamp();
            let old_stamp = old_belief.stamp();
            // 若为「重复任务」——优先级放到最后
            if new_stamp.equals(old_stamp) {
                if task.parent_task().as_ref().unwrap().is_judgement() {
                    task.budget_mut().dec_priority(C::ShortFloat::ZERO);
                }
                return;
            } else if <Self::DerivationContextDirect as LocalRules<C>>::revisable(
                judgement, old_belief,
            ) {
                *context.new_stamp_mut() =
                    <C::Stamp as StampConcrete>::from_merge(new_stamp, old_stamp, context.time());
                if context.new_stamp().is_some() {
                    // 🆕此处复制了「旧信念」以便设置值
                    // TODO: ❓是否需要这样：有可能后续处在「概念」中的信念被修改了，这里所指向的「信念」却没有
                    *context.current_belief_mut() = Some(old_belief.clone());
                    let old_belief = context.current_belief().as_ref().unwrap();
                    let old_belief = &old_belief.clone();
                    // ! 📌依靠复制，牺牲性能以**解决引用问题**（不然会引用`context`）
                    // * ❓↑但，这样会不会受到影响
                    LocalRulesDirect::revision(context, judgement, old_belief);
                }
            }
        }
        if task
            .budget()
            .above_threshold(ShortFloat::from_float(DEFAULT_PARAMETERS.budget_threshold))
        {
            for question in concept.__questions() {
                context.try_solution(judgement, question);
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
        context: &mut Self::DerivationContextDirect,
        concept: &mut C::Concept,
        task: &mut C::Task,
    ) -> C::ShortFloat {
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
        let new_answer = Self::__evaluation(question, concept.__beliefs());
        if let Some(new_answer) = new_answer {
            LocalRules::try_solution(context, new_answer, task);
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
        C::ShortFloat::from_float(result)
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
    fn __evaluation<'l>(query: &C::Sentence, list: &'l [C::Sentence]) -> Option<&'l C::Sentence> {
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
            .max_by_key(|judgement| {
                <Self::DerivationContextDirect as LocalRules<C>>::solution_quality(
                    Some(query),
                    judgement,
                )
            });
        candidate
    }
}

/// 通过「批量实现」自动加功能
impl<C: ReasonContext, T: Reasoner<C>> ReasonerDirectProcess<C> for T {}
