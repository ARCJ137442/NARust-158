//! 🎯复刻OpenNARS `nars.inference.SyllogisticRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::reason_context)
//!
//! * ✅【2024-05-11 10:08:34】初步复现方法API

use super::ReasonContext;
use crate::{entity::*, inference::*, language::Term};

/// 🆕表示「三段论侧」
/// * 🎯使用枚举对标以下推理中的`side`参数
///   * 起因：无法单纯使用[`SyllogismPosition`]
/// * 🚩三种情况：
///   * 主项
///   * 谓项
///   * 整体
///
/// # 📄OpenNARS
///
/// The location of the shared term in premise2:
/// - 0 for subject,
/// - 1 for predicate,
/// - -1 for the whole term
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyllogismSide {
    /// 主项（第一项）
    Subject = 0,
    /// 谓项（第二项）
    Predicate = 1,
    /// 整体（整个词项）
    Whole = -1,
}

/// 模拟`SyllogisticRules`
/// * 📝这些规则均为与「陈述」直接相关的规则
///   * 📌许多均以「陈述」为主体，出现在「条件」或「结论」中
///   * 📌包含从NAL-1到NAL-6的规则
///
/// # 📄OpenNARS
///
/// Syllogisms: Inference rules based on the transitivity of the relation.
pub trait SyllogisticRules: ReasonContext {
    // --------------- rules used in both first-tense inference and higher-tense inference ---------------

    /// 模拟`SyllogisticRules.dedExe`
    /// * 📝演绎 deduction & 举例 exemplification
    ///
    /// # 📄OpenNARS
    ///
    /// ```narsese
    /// {<S ==> M>, <M ==> P>} |- {<S ==> P>, <P ==> S>}
    /// ```
    ///
    /// @param term1    Subject of the first new task
    /// @param term2    Predicate of the first new task
    /// @param sentence The first premise
    /// @param belief   The second premise
    /// @param memory   Reference to the memory
    fn ded_exe(
        term1: &Term,
        term2: &Term,
        sentence: &Self::Sentence,
        belief: &Self::Sentence,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (Statement.invalidStatement(term1, term2)) {
            return;
        }
        TruthValue value1 = sentence.getTruth();
        TruthValue value2 = belief.getTruth();
        TruthValue truth1 = null;
        TruthValue truth2 = null;
        BudgetValue budget1, budget2;
        if (sentence.isQuestion()) {
            budget1 = BudgetFunctions.backwardWeak(value2, memory);
            budget2 = BudgetFunctions.backwardWeak(value2, memory);
        } else {
            truth1 = TruthFunctions.deduction(value1, value2);
            truth2 = TruthFunctions.exemplification(value1, value2);
            budget1 = BudgetFunctions.forward(truth1, memory);
            budget2 = BudgetFunctions.forward(truth2, memory);
        }
        Statement content = (Statement) sentence.getContent();
        Statement content1 = Statement.make(content, term1, term2, memory);
        Statement content2 = Statement.make(content, term2, term1, memory);
        memory.doublePremiseTask(content1, truth1, budget1);
        memory.doublePremiseTask(content2, truth2, budget2); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.abdIndCom`
    /// * 📝归因 abduction & 归纳 induction & 比较 comparison
    ///
    /// # 📄OpenNARS
    ///
    /// {<M ==> S>, <M ==> P>} |- {<S ==> P>, <P ==> S>, <S <=> P>}
    ///
    /// @param term1        Subject of the first new task
    /// @param term2        Predicate of the first new task
    /// @param taskSentence The first premise
    /// @param belief       The second premise
    /// @param figure       Locations of the shared term in premises
    /// @param memory       Reference to the memory
    fn abd_ind_com(
        term1: &Term,
        term2: &Term,
        task_sentence: &Self::Sentence,
        belief: &Self::Sentence,
        figure: &SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (Statement.invalidStatement(term1, term2) || Statement.invalidPair(term1.getName(), term2.getName())) {
            return;
        }
        Statement taskContent = (Statement) taskSentence.getContent();
        TruthValue truth1 = null;
        TruthValue truth2 = null;
        TruthValue truth3 = null;
        BudgetValue budget1, budget2, budget3;
        TruthValue value1 = taskSentence.getTruth();
        TruthValue value2 = belief.getTruth();
        if (taskSentence.isQuestion()) {
            budget1 = BudgetFunctions.backward(value2, memory);
            budget2 = BudgetFunctions.backwardWeak(value2, memory);
            budget3 = BudgetFunctions.backward(value2, memory);
        } else {
            truth1 = TruthFunctions.abduction(value1, value2);
            truth2 = TruthFunctions.abduction(value2, value1);
            truth3 = TruthFunctions.comparison(value1, value2);
            budget1 = BudgetFunctions.forward(truth1, memory);
            budget2 = BudgetFunctions.forward(truth2, memory);
            budget3 = BudgetFunctions.forward(truth3, memory);
        }
        Statement statement1 = Statement.make(taskContent, term1, term2, memory);
        Statement statement2 = Statement.make(taskContent, term2, term1, memory);
        Statement statement3 = Statement.makeSym(taskContent, term1, term2, memory);
        memory.doublePremiseTask(statement1, truth1, budget1);
        memory.doublePremiseTask(statement2, truth2, budget2);
        memory.doublePremiseTask(statement3, truth3, budget3); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.analogy`
    /// * 📝类比推理
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> P>, <M <=> P>} |- <S ==> P>
    ///
    /// @param subj       Subject of the new task
    /// @param pred       Predicate of the new task
    /// @param asymmetric The asymmetric premise
    /// @param symmetric  The symmetric premise
    /// @param figure     Locations of the shared term in premises
    /// @param memory     Reference to the memory
    fn analogy(
        subject: &Term,
        predicate: &Term,
        asym: &Self::Sentence,
        sym: &Self::Sentence,
        figure: SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (Statement.invalidStatement(subj, pred)) {
            return;
        }
        Statement st = (Statement) asym.getContent();
        TruthValue truth = null;
        BudgetValue budget;
        Sentence sentence = memory.currentTask.getSentence();
        CompoundTerm taskTerm = (CompoundTerm) sentence.getContent();
        if (sentence.isQuestion()) {
            if (taskTerm.isCommutative()) {
                budget = BudgetFunctions.backwardWeak(asym.getTruth(), memory);
            } else {
                budget = BudgetFunctions.backward(sym.getTruth(), memory);
            }
        } else {
            truth = TruthFunctions.analogy(asym.getTruth(), sym.getTruth());
            budget = BudgetFunctions.forward(truth, memory);
        }
        Term content = Statement.make(st, subj, pred, memory);
        memory.doublePremiseTask(content, truth, budget); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.resemblance`
    /// * 📝「相似/等价」的传递性
    ///
    /// # 📄OpenNARS
    ///
    ///  {<S <=> M>, <M <=> P>} |- <S <=> P>
    ///
    /// @param term1    Subject of the new task
    /// @param term2    Predicate of the new task
    /// @param belief   The first premise
    /// @param sentence The second premise
    /// @param figure   Locations of the shared term in premises
    /// @param memory   Reference to the memory
    fn resemblance(
        subject: &Term,
        predicate: &Term,
        belief: &Self::Sentence,
        sentence: &Self::Sentence,
        figure: SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (Statement.invalidStatement(term1, term2)) {
            return;
        }
        Statement st = (Statement) belief.getContent();
        TruthValue truth = null;
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.backward(belief.getTruth(), memory);
        } else {
            truth = TruthFunctions.resemblance(belief.getTruth(), sentence.getTruth());
            budget = BudgetFunctions.forward(truth, memory);
        }
        Term statement = Statement.make(st, term1, term2, memory);
        memory.doublePremiseTask(statement, truth, budget); */
        todo!("// TODO: 有待复现")
    }

    /* --------------- rules used only in conditional inference --------------- */

    /// 模拟`SyllogisticRules.detachment`
    /// * 🚩🆕【2024-05-11 09:46:49】其中的`side`使用[`SyllogismPosition`]代替
    ///
    /// # 📄OpenNARS
    ///
    /// {<<M --> S> ==> <M --> P>>, <M --> S>} |- <M --> P>
    /// {<<M --> S> ==> <M --> P>>, <M --> P>} |- <M --> S>
    /// {<<M --> S> <=> <M --> P>>, <M --> S>} |- <M --> P>
    /// {<<M --> S> <=> <M --> P>>, <M --> P>} |- <M --> S>
    ///
    /// @param mainSentence The implication/equivalence premise
    /// @param subSentence  The premise on part of s1
    /// @param side         The location of s2 in s1
    /// @param memory       Reference to the memory
    fn detachment(
        main_sentence: &Self::Sentence,
        sub_sentence: &Self::Sentence,
        side: SyllogismSide,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Statement statement = (Statement) mainSentence.getContent();
        if (!(statement instanceof Implication) && !(statement instanceof Equivalence)) {
            return;
        }
        Term subject = statement.getSubject();
        Term predicate = statement.getPredicate();
        Term content;
        Term term = subSentence.getContent();
        if ((side == 0) && term.equals(subject)) {
            content = predicate;
        } else if ((side == 1) && term.equals(predicate)) {
            content = subject;
        } else {
            return;
        }
        if ((content instanceof Statement) && ((Statement) content).invalid()) {
            return;
        }
        Sentence taskSentence = memory.currentTask.getSentence();
        Sentence beliefSentence = memory.currentBelief;
        TruthValue beliefTruth = beliefSentence.getTruth();
        TruthValue truth1 = mainSentence.getTruth();
        TruthValue truth2 = subSentence.getTruth();
        TruthValue truth = null;
        BudgetValue budget;
        if (taskSentence.isQuestion()) {
            if (statement instanceof Equivalence) {
                budget = BudgetFunctions.backward(beliefTruth, memory);
            } else if (side == 0) {
                budget = BudgetFunctions.backwardWeak(beliefTruth, memory);
            } else {
                budget = BudgetFunctions.backward(beliefTruth, memory);
            }
        } else {
            if (statement instanceof Equivalence) {
                truth = TruthFunctions.analogy(truth2, truth1);
            } else if (side == 0) {
                truth = TruthFunctions.deduction(truth1, truth2);
            } else {
                truth = TruthFunctions.abduction(truth2, truth1);
            }
            budget = BudgetFunctions.forward(truth, memory);
        }
        memory.doublePremiseTask(content, truth, budget); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.conditionalDedInd`
    /// * 📝带条件演绎、归纳
    /// * 🚩【2024-05-11 09:56:50】此处无法在编译期假定`premise1`为「蕴含」
    ///
    /// # 📄OpenNARS
    ///
    /// {<(&&, S1, S2, S3) ==> P>, S1} |- <(&&, S2, S3) ==> P>
    /// {<(&&, S2, S3) ==> P>, <S1 ==> S2>} |- <(&&, S1, S3) ==> P>
    /// {<(&&, S1, S3) ==> P>, <S1 ==> S2>} |- <(&&, S2, S3) ==> P>
    ///
    /// @param premise1 The conditional premise
    /// @param index    The location of the shared term in the condition of premise1
    /// @param premise2 The premise which, or part of which, appears in the condition of premise1
    /// @param side     The location of the shared term in premise2: 0 for subject, 1 for predicate, -1 for the whole term
    /// @param memory   Reference to the memory
    fn conditional_ded_ind(
        premise1: &Self::Sentence,
        index: SyllogismPosition,
        premise2: &Term,
        side: SyllogismSide,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence taskSentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        boolean deduction = (side != 0);
        boolean conditionalTask = Variable.hasSubstitute(Symbols.VAR_INDEPENDENT, premise2, belief.getContent());
        Term commonComponent;
        Term newComponent = null;
        if (side == 0) {
            commonComponent = ((Statement) premise2).getSubject();
            newComponent = ((Statement) premise2).getPredicate();
        } else if (side == 1) {
            commonComponent = ((Statement) premise2).getPredicate();
            newComponent = ((Statement) premise2).getSubject();
        } else {
            commonComponent = premise2;
        }
        Term subj = premise1.getSubject();
        if (!(subj instanceof Conjunction)) {
            return;
        }
        Conjunction oldCondition = (Conjunction) subj;
        int index2 = oldCondition.getComponents().indexOf(commonComponent);
        if (index2 >= 0) {
            index = (short) index2;
        } else {
            boolean match = Variable.unify(Symbols.VAR_INDEPENDENT, oldCondition.componentAt(index), commonComponent,
                    premise1, premise2);
            if (!match && (commonComponent.getClass() == oldCondition.getClass())) {
                match = Variable.unify(Symbols.VAR_INDEPENDENT, oldCondition.componentAt(index),
                        ((CompoundTerm) commonComponent).componentAt(index), premise1, premise2);
            }
            if (!match) {
                return;
            }
        }
        Term newCondition;
        if (oldCondition.equals(commonComponent)) {
            newCondition = null;
        } else {
            newCondition = CompoundTerm.setComponent(oldCondition, index, newComponent, memory);
        }
        Term content;
        if (newCondition != null) {
            content = Statement.make(premise1, newCondition, premise1.getPredicate(), memory);
        } else {
            content = premise1.getPredicate();
        }
        if (content == null) {
            return;
        }
        TruthValue truth1 = taskSentence.getTruth();
        TruthValue truth2 = belief.getTruth();
        TruthValue truth = null;
        BudgetValue budget;
        if (taskSentence.isQuestion()) {
            budget = BudgetFunctions.backwardWeak(truth2, memory);
        } else {
            if (deduction) {
                truth = TruthFunctions.deduction(truth1, truth2);
            } else if (conditionalTask) {
                truth = TruthFunctions.induction(truth2, truth1);
            } else {
                truth = TruthFunctions.induction(truth1, truth2);
            }
            budget = BudgetFunctions.forward(truth, memory);
        }
        memory.doublePremiseTask(content, truth, budget); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.conditionalAna`
    /// * 📝带条件类比 analogy
    ///
    /// # 📄OpenNARS
    ///
    /// {<(&&, S1, S2) <=> P>, (&&, S1, S2)} |- P
    ///
    /// @param premise1 The equivalence premise
    /// @param index    The location of the shared term in the condition of premise1
    /// @param premise2 The premise which, or part of which, appears in the condition of premise1
    /// @param side     The location of the shared term in premise2: 0 for subject, 1 for predicate, -1 for the whole term
    /// @param memory   Reference to the memory
    fn conditional_ana(
        premise1: &Self::Sentence,
        index: SyllogismPosition,
        premise2: &Term,
        side: SyllogismSide,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence taskSentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        boolean conditionalTask = Variable.hasSubstitute(Symbols.VAR_INDEPENDENT, premise2, belief.getContent());
        Term commonComponent;
        Term newComponent = null;
        if (side == 0) {
            commonComponent = ((Statement) premise2).getSubject();
            newComponent = ((Statement) premise2).getPredicate();
        } else if (side == 1) {
            commonComponent = ((Statement) premise2).getPredicate();
            newComponent = ((Statement) premise2).getSubject();
        } else {
            commonComponent = premise2;
        }
        Term tm = premise1.getSubject();
        if (!(tm instanceof Conjunction))
            return;
        Conjunction oldCondition = (Conjunction) tm;
        boolean match = Variable.unify(Symbols.VAR_DEPENDENT, oldCondition.componentAt(index), commonComponent,
                premise1, premise2);
        if (!match && (commonComponent.getClass() == oldCondition.getClass())) {
            match = Variable.unify(Symbols.VAR_DEPENDENT, oldCondition.componentAt(index),
                    ((CompoundTerm) commonComponent).componentAt(index), premise1, premise2);
        }
        if (!match) {
            return;
        }
        Term newCondition;
        if (oldCondition.equals(commonComponent)) {
            newCondition = null;
        } else {
            newCondition = CompoundTerm.setComponent(oldCondition, index, newComponent, memory);
        }
        Term content;
        if (newCondition != null) {
            content = Statement.make(premise1, newCondition, premise1.getPredicate(), memory);
        } else {
            content = premise1.getPredicate();
        }
        if (content == null) {
            return;
        }
        TruthValue truth1 = taskSentence.getTruth();
        TruthValue truth2 = belief.getTruth();
        TruthValue truth = null;
        BudgetValue budget;
        if (taskSentence.isQuestion()) {
            budget = BudgetFunctions.backwardWeak(truth2, memory);
        } else {
            if (conditionalTask) {
                truth = TruthFunctions.comparison(truth1, truth2);
            } else {
                truth = TruthFunctions.analogy(truth1, truth2);
            }
            budget = BudgetFunctions.forward(truth, memory);
        }
        memory.doublePremiseTask(content, truth, budget); */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.conditionalAbd`
    /// * 📝条件归因 abduction
    /// * ❌【2024-05-11 10:03:47】无法假定其中的`st1`、`st2`为「陈述」
    ///
    /// # 📄OpenNARS
    ///
    /// {<(&&, S2, S3) ==> P>, <(&&, S1, S3) ==> P>} |- <S1 ==> S2>
    ///
    /// @param cond1       The condition of the first premise
    /// @param cond2       The condition of the second premise
    /// @param taskContent The first premise
    /// @param st2         The second premise
    /// @param memory      Reference to the memory
    /// @return Whether there are derived tasks
    fn conditional_abd(cond1: &Term, cond2: &Term, st1: &Term, st2: &Term) {
        /* 📄OpenNARS源码：
        if (!(st1 instanceof Implication) || !(st2 instanceof Implication)) {
            return false;
        }
        if (!(cond1 instanceof Conjunction) && !(cond2 instanceof Conjunction)) {
            return false;
        }
        Term term1 = null;
        Term term2 = null;
        // if ((cond1 instanceof Conjunction) &&
        // !Variable.containVarDep(cond1.getName())) {
        if (cond1 instanceof Conjunction) {
            term1 = CompoundTerm.reduceComponents((Conjunction) cond1, cond2, memory);
        }
        // if ((cond2 instanceof Conjunction) &&
        // !Variable.containVarDep(cond2.getName())) {
        if (cond2 instanceof Conjunction) {
            term2 = CompoundTerm.reduceComponents((Conjunction) cond2, cond1, memory);
        }
        if ((term1 == null) && (term2 == null)) {
            return false;
        }
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        TruthValue value1 = sentence.getTruth();
        TruthValue value2 = belief.getTruth();
        Term content;
        TruthValue truth = null;
        BudgetValue budget;
        if (term1 != null) {
            if (term2 != null) {
                content = Statement.make(st2, term2, term1, memory);
            } else {
                content = term1;
            }
            if (sentence.isQuestion()) {
                budget = BudgetFunctions.backwardWeak(value2, memory);
            } else {
                truth = TruthFunctions.abduction(value2, value1);
                budget = BudgetFunctions.forward(truth, memory);
            }
            memory.doublePremiseTask(content, truth, budget);
        }
        if (term2 != null) {
            if (term1 != null) {
                content = Statement.make(st1, term1, term2, memory);
            } else {
                content = term2;
            }
            if (sentence.isQuestion()) {
                budget = BudgetFunctions.backwardWeak(value2, memory);
            } else {
                truth = TruthFunctions.abduction(value1, value2);
                budget = BudgetFunctions.forward(truth, memory);
            }
            memory.doublePremiseTask(content, truth, budget);
        }
        return true; */
        todo!("// TODO: 有待复现")
    }

    /// 模拟`SyllogisticRules.eliminateVarDep`
    ///
    /// # 📄OpenNARS
    ///
    /// {(&&, <#x() --> S>, <#x() --> P>>, <M --> P>} |- <M --> S>
    ///
    /// @param compound     The compound term to be decomposed
    /// @param component    The part of the compound to be removed
    /// @param compoundTask Whether the compound comes from the task
    /// @param memory       Reference to the memory
    fn eliminate_var_dep(
        compound: &Term,
        component: &Term,
        compound_task: &Self::Task,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Term content = CompoundTerm.reduceComponents(compound, component, memory);
        if ((content == null) || ((content instanceof Statement) && ((Statement) content).invalid()))
            return;
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        TruthValue v1 = sentence.getTruth();
        TruthValue v2 = belief.getTruth();
        TruthValue truth = null;
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = (compoundTask ? BudgetFunctions.backward(v2, memory) : BudgetFunctions.backwardWeak(v2, memory));
        } else {
            truth = (compoundTask ? TruthFunctions.anonymousAnalogy(v1, v2) : TruthFunctions.anonymousAnalogy(v2, v1));
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.doublePremiseTask(content, truth, budget); */
        todo!("// TODO: 有待复现")
    }
}

/// 自动实现，以便添加方法
impl<T: ReasonContext> SyllogisticRules for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
