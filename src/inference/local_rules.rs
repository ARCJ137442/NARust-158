//! 🎯复刻OpenNARS `nars.inference.LocalRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::reason_context)
//! * ✅【2024-05-07 18:51:30】初步实现方法API（函数签名、文档、源码附注）

use crate::{entity::*, inference::*, io::symbols::VAR_QUERY, language::variable::unify_two};

/// 模拟`LocalRules`
/// * 📝有关「内部思考」「内省」的规则
///   * 📄真值修正
///   * 📄问题回答
///   * 📄命题转换（部分？）
///
/// # 📄OpenNARS
///
/// Directly process a task by a oldBelief, with only two Terms in both. In
/// matching, the new task is compared with all existing direct Tasks in that
/// Concept, to carry out:
///
/// revision: between judgments on non-overlapping evidence; revision: between
/// judgments; satisfy: between a Sentence and a Question/Goal; merge: between
/// items of the same type and stamp; conversion: between different inheritance
/// relations.
pub trait LocalRules: DerivationContext {
    /* -------------------- same contents -------------------- */

    /// 模拟`LocalRules.match`
    /// * 🚩【2024-05-07 18:09:32】避开关键词，改为`match_belief`
    ///
    /// # 📄OpenNARS
    ///
    /// The task and belief have the same content
    ///
    /// called in RuleTables.reason
    ///
    /// @param task   The task
    /// @param belief The belief
    /// @param memory Reference to the memory
    #[doc(alias = "match")]
    fn match_belief(&mut self, task: &Self::Task, belief: &Self::Sentence) {
        /* 📄OpenNARS源码：
        Sentence sentence = (Sentence) task.getSentence().clone();
        if (sentence.isJudgment()) {
            if (revisable(sentence, belief)) {
                revision(sentence, belief, true, memory);
            }
        } else if (Variable.unify(Symbols.VAR_QUERY, sentence.getContent(), (Term) belief.getContent().clone())) {
            trySolution(belief, task, memory);
        } */
        let sentence = task.sentence();
        match sentence.punctuation() {
            // 判断⇒若能修订，转换到「修订」
            SentenceType::Judgement(..) => {
                if <Self as LocalRules>::revisable(sentence, belief) {
                    self.revision(sentence, belief, true);
                }
            }
            // 问题⇒尝试用信念解答
            SentenceType::Question => {
                if unify_two(
                    VAR_QUERY,
                    // ! 🚩OpenNARS原意即「复制出一个新语句」而非直接修改
                    task.sentence().clone().content_mut(),
                    &mut belief.content().clone(),
                ) {
                    self.try_solution(belief, task);
                }
            }
        }
    }

    /// 模拟`LocalRules.revisable`
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether two sentences can be used in revision
    ///
    /// @param s1 The first sentence
    /// @param s2 The second sentence
    /// @return If revision is possible between the two sentences
    fn revisable(s1: &Self::Sentence, s2: &Self::Sentence) -> bool {
        /* 📄OpenNARS源码：
        return (s1.getContent().equals(s2.getContent()) && s1.getRevisable()); */
        s1.content() == s2.content() && s1.revisable()
    }

    /// 模拟`LocalRules.revision`
    ///
    /// # 📄OpenNARS
    ///
    /// Belief revision
    ///
    /// called from Concept.reviseTable and match
    ///
    /// @param newBelief       The new belief in task
    /// @param oldBelief       The previous belief with the same content
    /// @param feedbackToLinks Whether to send feedback to the links
    /// @param memory          Reference to the memory
    fn revision(
        &mut self,
        new_belief: &Self::Sentence,
        old_belief: &Self::Sentence,
        feedback_to_links: bool,
    ) {
        /* 📄OpenNARS源码：
        TruthValue newTruth = newBelief.getTruth();
        TruthValue oldTruth = oldBelief.getTruth();
        TruthValue truth = TruthFunctions.revision(newTruth, oldTruth);
        BudgetValue budget = BudgetFunctions.revise(newTruth, oldTruth, truth, feedbackToLinks, memory);
        Term content = newBelief.getContent();
        memory.doublePremiseTask(content, truth, budget); */
        // ! 此处必须假定俩语句有「真值」
        let new_truth = new_belief.truth().unwrap();
        let old_truth = old_belief.truth().unwrap();
        let truth = new_truth.revision(old_truth);
        // ! 此处真的要修改词项链、任务链
        let budget = <<Self as ReasonContext>::Budget>::revise(
            new_truth,
            old_truth,
            &truth,
            feedback_to_links,
            self,
        );
        let content = new_belief.content();
        self.double_premise_task_revisable(content.clone(), truth, budget);
    }

    /// 模拟`LocalRules.trySolution`
    ///
    /// # 📄OpenNARS
    ///
    /// Check if a Sentence provide a better answer to a Question or Goal
    ///
    /// @param belief The proposed answer
    /// @param task   The task to be processed
    /// @param memory Reference to the memory
    fn try_solution(&mut self, belief: &Self::Sentence, task: &Self::Task) {
        /* 📄OpenNARS源码：
        Sentence problem = task.getSentence();
        Sentence oldBest = task.getBestSolution();
        float newQ = solutionQuality(problem, belief);
        if (oldBest != null) {
            float oldQ = solutionQuality(problem, oldBest);
            if (oldQ >= newQ) {
                return;
            }
        }
        task.setBestSolution(belief);
        if (task.isInput()) { // moved from Sentence
            memory.report(belief, Memory.ReportType.ANSWER);
        }
        BudgetValue budget = BudgetFunctions.solutionEval(problem, belief, task, memory);
        if ((budget != null) && budget.aboveThreshold()) {
            memory.activatedTask(budget, belief, task.getParentBelief());
        } */
    }

    /// 模拟`LocalRules.solutionQuality`
    ///
    /// # 📄OpenNARS
    ///
    /// Evaluate the quality of the judgment as a solution to a problem
    ///
    /// @param problem  A goal or question
    /// @param solution The solution to be evaluated
    /// @return The quality of the judgment as the solution
    fn solution_quality(
        problem: Option<&Self::Sentence>,
        solution: &Self::Sentence,
    ) -> Self::ShortFloat {
        /* 📄OpenNARS源码：
        if (problem == null) {
            return solution.getTruth().getExpectation();
        }
        TruthValue truth = solution.getTruth();
        if (problem.containQueryVar()) { // "yes/no" question
            return truth.getExpectation() / solution.getContent().getComplexity();
        } else { // "what" question or goal
            return truth.getConfidence();
        } */
        let s_truth = solution.truth().unwrap();
        match problem {
            None => <Self::ShortFloat as ShortFloat>::from_float(s_truth.expectation()),
            Some(problem) => match problem.contain_query_var() {
                true => <Self::ShortFloat as ShortFloat>::from_float(s_truth.expectation()),
                false => s_truth.confidence(),
            },
        }
    }

    /* -------------------- same terms, difference relations -------------------- */

    /// 模拟`LocalRules.matchReverse`
    ///
    /// # 📄OpenNARS
    ///
    /// The task and belief match reversely
    ///
    /// @param memory Reference to the memory
    fn match_reverse(&mut self) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence belief = memory.currentBelief;
        Sentence sentence = task.getSentence();
        if (sentence.isJudgment()) {
            inferToSym((Sentence) sentence, belief, memory);
        } else {
            conversion(memory);
        } */
    }

    /// 模拟`LocalRules.matchAsymSym`
    ///
    /// # 📄OpenNARS
    ///
    /// Inheritance/Implication matches Similarity/Equivalence
    ///
    /// @param asym   A Inheritance/Implication sentence
    /// @param sym    A Similarity/Equivalence sentence
    /// @param figure location of the shared term
    /// @param memory Reference to the memory
    fn match_asym_sym(
        &mut self,
        asym: &Self::Sentence,
        sym: &Self::Sentence,
        figure: SyllogismFigure,
    ) {
        /* 📄OpenNARS源码：
        if (memory.currentTask.getSentence().isJudgment()) {
            inferToAsym((Sentence) asym, (Sentence) sym, memory);
        } else {
            convertRelation(memory);
        } */
    }

    /* -------------------- two-premise inference rules -------------------- */

    /// 模拟`LocalRules.inferToSym`
    ///
    /// # 📄OpenNARS
    ///
    /// {<S --> P>, <P --> S} |- <S <-> p> Produce Similarity/Equivalence from a
    /// pair of reversed Inheritance/Implication
    ///
    /// @param judgment1 The first premise
    /// @param judgment2 The second premise
    /// @param memory    Reference to the memory
    fn __infer_to_sym(&mut self, judgement1: &Self::Sentence, judgement2: &Self::Sentence) {
        /* 📄OpenNARS源码：
        Statement s1 = (Statement) judgment1.getContent();
        Term t1 = s1.getSubject();
        Term t2 = s1.getPredicate();
        Term content;
        if (s1 instanceof Inheritance) {
            content = Similarity.make(t1, t2, memory);
        } else {
            content = Equivalence.make(t1, t2, memory);
        }
        TruthValue value1 = judgment1.getTruth();
        TruthValue value2 = judgment2.getTruth();
        TruthValue truth = TruthFunctions.intersection(value1, value2);
        BudgetValue budget = BudgetFunctions.forward(truth, memory);
        memory.doublePremiseTask(content, truth, budget); */
    }

    /// 模拟`LocalRules.inferToAsym`
    ///
    /// # 📄OpenNARS
    ///
    /// {<S <-> P>, <P --> S>} |- <S --> P> Produce an Inheritance/Implication
    /// from a Similarity/Equivalence and a reversed Inheritance/Implication
    ///
    /// @param asym   The asymmetric premise
    /// @param sym    The symmetric premise
    /// @param memory Reference to the memory
    fn __infer_to_asym(&mut self, asym: &Self::Task, sym: &Self::Sentence) {
        /* 📄OpenNARS源码：
        Statement statement = (Statement) asym.getContent();
        Term sub = statement.getPredicate();
        Term pre = statement.getSubject();
        Statement content = Statement.make(statement, sub, pre, memory);
        TruthValue truth = TruthFunctions.reduceConjunction(sym.getTruth(), asym.getTruth());
        BudgetValue budget = BudgetFunctions.forward(truth, memory);
        memory.doublePremiseTask(content, truth, budget); */
    }

    /* -------------------- one-premise inference rules -------------------- */

    /// 模拟`LocalRules.conversion`
    ///
    /// # 📄OpenNARS
    ///
    /// {<P --> S>} |- <S --> P> Produce an Inheritance/Implication from a
    /// reversed Inheritance/Implication
    ///
    /// @param memory Reference to the memory
    fn __conversion(&mut self) {
        /* 📄OpenNARS源码：
        TruthValue truth = TruthFunctions.conversion(memory.currentBelief.getTruth());
        BudgetValue budget = BudgetFunctions.forward(truth, memory);
        convertedJudgment(truth, budget, memory); */
    }

    /// 模拟`LocalRules.convertRelation`
    ///
    /// # 📄OpenNARS
    ///
    /// {<S --> P>} |- <S <-> P> {<S <-> P>} |- <S --> P> Switch between
    /// Inheritance/Implication and Similarity/Equivalence
    ///
    /// @param memory Reference to the memory
    fn __convert_relation(&mut self) {
        /* 📄OpenNARS源码：
        TruthValue truth = memory.currentBelief.getTruth();
        if (((Statement) memory.currentTask.getContent()).isCommutative()) {
            truth = TruthFunctions.abduction(truth, 1.0f);
        } else {
            truth = TruthFunctions.deduction(truth, 1.0f);
        }
        BudgetValue budget = BudgetFunctions.forward(truth, memory);
        convertedJudgment(truth, budget, memory); */
    }

    /// 模拟`LocalRules.convertedJudgment`
    ///
    /// # 📄OpenNARS
    fn __converted_judgment(&mut self, new_truth: &Self::Truth, new_budget: &Self::Budget) {
        /* 📄OpenNARS源码：
        Statement content = (Statement) memory.currentTask.getContent();
        Statement beliefContent = (Statement) memory.currentBelief.getContent();
        Term subjT = content.getSubject();
        Term predT = content.getPredicate();
        Term subjB = beliefContent.getSubject();
        Term predB = beliefContent.getPredicate();
        Term otherTerm;
        if (Variable.containVarQ(subjT.getName())) {
            otherTerm = (predT.equals(subjB)) ? predB : subjB;
            content = Statement.make(content, otherTerm, predT, memory);
        }
        if (Variable.containVarQ(predT.getName())) {
            otherTerm = (subjT.equals(subjB)) ? predB : subjB;
            content = Statement.make(content, subjT, otherTerm, memory);
        }
        memory.singlePremiseTask(content, Symbols.JUDGMENT_MARK, newTruth, newBudget); */
    }
}

/// 自动实现，以便添加方法
impl<T: DerivationContext> LocalRules for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
