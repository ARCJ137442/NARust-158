//! 「概念处理」主模块
//! * 🎯有关「概念推理」的主控
//!   * 📌信念获取 from 组合规则、规则表
//!   * 📌添加入表 from 处理判断
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部) from 直接处理
//!   * 📌处理问题(内部) from 直接处理
//!   * 📌「点火」 from 处理概念(@记忆区)
//!
//! * ♻️【2024-05-16 18:07:08】初步独立成模块功能

use crate::global::ClockTime;
use crate::inference::{DerivationContext, LocalRules};
use crate::{entity::*, inference::*, nars::DEFAULT_PARAMETERS};

/// 有关「概念」的处理
/// * 🎯分离NARS控制机制中有关「概念」的部分
/// * 📌此处均有关「直接推理」
///   * 📝OpenNARS中均由`Memory.immediateProcess`调用
pub trait ConceptProcessDirect<C: ReasonContext>: DerivationContextDirect<C> {
    /* ---------- direct processing of tasks ---------- */

    /// 模拟`Concept.getBelief`
    /// * 📝OpenNARS用在「组合规则」与「推导上下文构建」中
    ///   * ✅「组合规则」中就是正常使用「推导上下文」：其「概念」就是「推理上下文」中使用到的「当前概念」
    ///   * ⚠️「推导上下文构建」中要同时获取「&mut 推导上下文」与「&概念」
    ///     * 🚩【2024-05-17 15:07:02】因此全部解耦：直接传引用
    /// * 🚩【2024-05-16 18:43:40】因为是「赋值『新时间戳』到上下文」，故需要`self`可变
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
        new_stamp_mut: &mut Option<C::Stamp>,
        time: ClockTime,
        concept: &C::Concept,
        task: &C::Task,
    ) -> Option<C::Sentence> {
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
            let new_stamp = C::Stamp::from_merge(task_sentence.stamp(), belief.stamp(), time);
            if new_stamp.is_some() {
                let belief2 = belief.clone();
                return Some(belief2);
            }
            // * 🚩必须赋值，无论是否有
            *new_stamp_mut = new_stamp;
        }
        None
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
    fn __add_to_table(sentence: &C::Sentence, table: &mut Vec<C::Sentence>, capacity: usize) {
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
    fn direct_process(&mut self, concept: &mut C::Concept, task: &mut C::Task) {
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
    fn __process_judgment(&mut self, concept: &C::Concept, task: &mut C::Task) {
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
                    task.budget_mut().dec_priority(C::ShortFloat::ZERO);
                }
                return;
            } else if <Self as LocalRules<C>>::revisable(judgement, old_belief) {
                *self.new_stamp_mut() =
                    <C::Stamp as StampConcrete>::from_merge(new_stamp, old_stamp, self.time());
                if self.new_stamp().is_some() {
                    // 🆕此处复制了「旧信念」以便设置值
                    // TODO: ❓是否需要这样：有可能后续处在「概念」中的信念被修改了，这里所指向的「信念」却没有
                    *self.current_belief_mut() = Some(old_belief.clone());
                    let old_belief = self.current_belief().as_ref().unwrap();
                    let old_belief = &old_belief.clone();
                    // ! 📌依靠复制，牺牲性能以**解决引用问题**（不然会引用`self`）
                    // * ❓↑但，这样会不会受到影响
                    LocalRulesDirect::revision(self, judgement, old_belief);
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
    fn __evaluation<'l>(
        &mut self,
        query: &C::Sentence,
        list: &'l [C::Sentence],
    ) -> Option<&'l C::Sentence> {
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
            .max_by_key(|judgement| Self::solution_quality(Some(query), judgement));
        candidate
    }
}

pub trait ConceptProcess<C: ReasonContext> {}

/// 自动实现，以便添加方法
impl<C: ReasonContext, T: DerivationContext<C>> ConceptProcess<C> for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
