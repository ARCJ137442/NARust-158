//! 🎯复刻OpenNARS `nars.inference.CompositionalRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::reason_context)
//!
//! * ✅【2024-05-12 00:47:43】初步复现方法API
//!
//! TODO: 🚧完成具体实现

use crate::inference::DerivationContext;
use crate::{entity::*, inference::*, language::Term};

/// 模拟`CompositionalRules`
/// * 📝这些规则基本与「复合词项」的有关
///   * 📌主要涉及「子项组合」「变量引入」等
///   * 💭注意和[`super::StructuralRules`]的异同
///
/// # 📄OpenNARS
///
/// Compound term composition and decomposition rules, with two premises.
///
/// Forward inference only, except the last group (dependent variable
/// introduction) can also be used backward.
pub trait CompositionalRules<C: ReasonContext> {
    /// 模拟`CompositionalRules.IntroVarSameSubjectOrPredicate`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn intro_var_same_subject_or_predicate(
        original_main_sentence: &C::Sentence,
        sub_sentence: &C::Sentence,
        component: &Term,
        content: &Term,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        Sentence cloned = (Sentence) originalMainSentence.clone();
        Term T1 = cloned.getContent();
        if (!(T1 instanceof CompoundTerm) || !(content instanceof CompoundTerm)) {
            return;
        }
        CompoundTerm T = (CompoundTerm) T1;
        CompoundTerm T2 = (CompoundTerm) content.clone();
        if ((component instanceof Inheritance && content instanceof Inheritance) ||
                (component instanceof Similarity && content instanceof Similarity)) {
            // CompoundTerm result = T;
            if (component.equals(content)) {
                // wouldn't make sense to create a conjunction here,
                // would contain a statement twice
                return;
            }
            if (((Statement) component).getPredicate().equals(((Statement) content).getPredicate())
                    && !(((Statement) component).getPredicate() instanceof Variable)) {
                Variable V = new Variable("#depIndVar1");
                CompoundTerm zw = (CompoundTerm) T.getComponents().get(index).clone();
                zw = (CompoundTerm) CompoundTerm.setComponent(zw, 1, V, memory);
                T2 = (CompoundTerm) CompoundTerm.setComponent(T2, 1, V, memory);
                if (zw == null || T2 == null || zw.equals(T2)) {
                    return;
                }
                Conjunction res = (Conjunction) Conjunction.make(zw, T2, memory);
                T = (CompoundTerm) CompoundTerm.setComponent(T, index, res, memory);
            } else if (((Statement) component).getSubject().equals(((Statement) content).getSubject())
                    && !(((Statement) component).getSubject() instanceof Variable)) {
                Variable V = new Variable("#depIndVar2");
                CompoundTerm zw = (CompoundTerm) T.getComponents().get(index).clone();
                zw = (CompoundTerm) CompoundTerm.setComponent(zw, 0, V, memory);
                T2 = (CompoundTerm) CompoundTerm.setComponent(T2, 0, V, memory);
                if (zw == null || T2 == null || zw.equals(T2)) {
                    return;
                }
                Conjunction res = (Conjunction) Conjunction.make(zw, T2, memory);
                T = (CompoundTerm) CompoundTerm.setComponent(T, index, res, memory);
            }
            TruthValue truth = TruthFunctions.induction(originalMainSentence.getTruth(), subSentence.getTruth());
            BudgetValue budget = BudgetFunctions.compoundForward(truth, T, memory);
            memory.doublePremiseTask(T, truth, budget); */
    }

    /* -------------------- intersections and differences -------------------- */

    /// 模拟`CompositionalRules.composeCompound`
    ///
    /// # 📄OpenNARS
    ///
    /// {<S ==> M>, <P ==> M>} |- {
    ///     <(S|P) ==> M>, <(S&P) ==> M>,
    ///     <(S-P) ==> M>, <(P-S) ==> M>
    /// }
    ///
    /// @param taskSentence The first premise
    /// @param belief       The second premise
    /// @param index        The location of the shared term
    /// @param memory       Reference to the memory
    fn compose_compound(
        task_statement: &Term,
        belief_statement: &Term,
        index: SyllogismPosition,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if ((!memory.currentTask.getSentence().isJudgment()) || (taskContent.getClass() != beliefContent.getClass())) {
            return;
        }
        Term componentT = taskContent.componentAt(1 - index);
        Term componentB = beliefContent.componentAt(1 - index);
        Term componentCommon = taskContent.componentAt(index);
        if ((componentT instanceof CompoundTerm) && ((CompoundTerm) componentT).containAllComponents(componentB)) {
            decomposeCompound((CompoundTerm) componentT, componentB, componentCommon, index, true, memory);
            return;
        } else if ((componentB instanceof CompoundTerm)
                && ((CompoundTerm) componentB).containAllComponents(componentT)) {
            decomposeCompound((CompoundTerm) componentB, componentT, componentCommon, index, false, memory);
            return;
        }
        TruthValue truthT = memory.currentTask.getSentence().getTruth();
        TruthValue truthB = memory.currentBelief.getTruth();
        TruthValue truthOr = TruthFunctions.union(truthT, truthB);
        TruthValue truthAnd = TruthFunctions.intersection(truthT, truthB);
        TruthValue truthDif = null;
        Term termOr = null;
        Term termAnd = null;
        Term termDif = null;
        if (index == 0) {
            if (taskContent instanceof Inheritance) {
                termOr = IntersectionInt.make(componentT, componentB, memory);
                termAnd = IntersectionExt.make(componentT, componentB, memory);
                if (truthB.isNegative()) {
                    if (!truthT.isNegative()) {
                        termDif = DifferenceExt.make(componentT, componentB, memory);
                        truthDif = TruthFunctions.intersection(truthT, TruthFunctions.negation(truthB));
                    }
                } else if (truthT.isNegative()) {
                    termDif = DifferenceExt.make(componentB, componentT, memory);
                    truthDif = TruthFunctions.intersection(truthB, TruthFunctions.negation(truthT));
                }
            } else if (taskContent instanceof Implication) {
                termOr = Disjunction.make(componentT, componentB, memory);
                termAnd = Conjunction.make(componentT, componentB, memory);
            }
            processComposed(taskContent, (Term) componentCommon.clone(), termOr, truthOr, memory);
            processComposed(taskContent, (Term) componentCommon.clone(), termAnd, truthAnd, memory);
            processComposed(taskContent, (Term) componentCommon.clone(), termDif, truthDif, memory);
        } else { // index == 1
            if (taskContent instanceof Inheritance) {
                termOr = IntersectionExt.make(componentT, componentB, memory);
                termAnd = IntersectionInt.make(componentT, componentB, memory);
                if (truthB.isNegative()) {
                    if (!truthT.isNegative()) {
                        termDif = DifferenceInt.make(componentT, componentB, memory);
                        truthDif = TruthFunctions.intersection(truthT, TruthFunctions.negation(truthB));
                    }
                } else if (truthT.isNegative()) {
                    termDif = DifferenceInt.make(componentB, componentT, memory);
                    truthDif = TruthFunctions.intersection(truthB, TruthFunctions.negation(truthT));
                }
            } else if (taskContent instanceof Implication) {
                termOr = Conjunction.make(componentT, componentB, memory);
                termAnd = Disjunction.make(componentT, componentB, memory);
            }
            processComposed(taskContent, termOr, (Term) componentCommon.clone(), truthOr, memory);
            processComposed(taskContent, termAnd, (Term) componentCommon.clone(), truthAnd, memory);
            processComposed(taskContent, termDif, (Term) componentCommon.clone(), truthDif, memory);
        }
        if (taskContent instanceof Inheritance) {
            introVarOuter(taskContent, beliefContent, index, memory);
            // introVarImage(taskContent, beliefContent, index, memory);
        } */
    }

    /// 模拟`CompositionalRules.processComposed`
    ///
    /// # 📄OpenNARS
    ///
    /// Finish composing implication term
    ///
    /// @param premise1  Type of the contentInd
    /// @param subject   Subject of contentInd
    /// @param predicate Predicate of contentInd
    /// @param truth     TruthValue of the contentInd
    /// @param memory    Reference to the memory
    fn __process_composed(
        statement: &Term,
        subject: &Term,
        predicate: &Term,
        truth: &C::Truth,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
         */
    }

    /// 模拟`CompositionalRules.decomposeCompound`
    ///
    /// # 📄OpenNARS
    ///
    /// {<(S|P) ==> M>, <P ==> M>} |- <S ==> M>
    ///
    /// @param implication     The implication term to be decomposed
    /// @param componentCommon The part of the implication to be removed
    /// @param term1           The other term in the contentInd
    /// @param index           The location of the shared term: 0 for subject, 1 for predicate
    /// @param compoundTask    Whether the implication comes from the task
    /// @param memory          Reference to the memory
    fn __decompose_compound(
        compound: &Term,
        component: &Term,
        term1: &Term,
        index: SyllogismPosition,
        compound_task: bool,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if ((compound instanceof Statement) || (compound instanceof ImageExt) || (compound instanceof ImageInt)) {
            return;
        }
        Term term2 = CompoundTerm.reduceComponents(compound, component, memory);
        if (term2 == null) {
            return;
        }
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        Statement oldContent = (Statement) task.getContent();
        TruthValue v1,
                v2;
        if (compoundTask) {
            v1 = sentence.getTruth();
            v2 = belief.getTruth();
        } else {
            v1 = belief.getTruth();
            v2 = sentence.getTruth();
        }
        TruthValue truth = null;
        Term content;
        if (index == 0) {
            content = Statement.make(oldContent, term1, term2, memory);
            if (content == null) {
                return;
            }
            if (oldContent instanceof Inheritance) {
                if (compound instanceof IntersectionExt) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if (compound instanceof IntersectionInt) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                } else if ((compound instanceof SetInt) && (component instanceof SetInt)) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if ((compound instanceof SetExt) && (component instanceof SetExt)) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                } else if (compound instanceof DifferenceExt) {
                    if (compound.componentAt(0).equals(component)) {
                        truth = TruthFunctions.reduceDisjunction(v2, v1);
                    } else {
                        truth = TruthFunctions.reduceConjunctionNeg(v1, v2);
                    }
                }
            } else if (oldContent instanceof Implication) {
                if (compound instanceof Conjunction) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if (compound instanceof Disjunction) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                }
            }
        } else {
            content = Statement.make(oldContent, term2, term1, memory);
            if (content == null) {
                return;
            }
            if (oldContent instanceof Inheritance) {
                if (compound instanceof IntersectionInt) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if (compound instanceof IntersectionExt) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                } else if ((compound instanceof SetExt) && (component instanceof SetExt)) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if ((compound instanceof SetInt) && (component instanceof SetInt)) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                } else if (compound instanceof DifferenceInt) {
                    if (compound.componentAt(1).equals(component)) {
                        truth = TruthFunctions.reduceDisjunction(v2, v1);
                    } else {
                        truth = TruthFunctions.reduceConjunctionNeg(v1, v2);
                    }
                }
            } else if (oldContent instanceof Implication) {
                if (compound instanceof Disjunction) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                } else if (compound instanceof Conjunction) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                }
            }
        }
        if (truth != null) {
            BudgetValue budget = BudgetFunctions.compoundForward(truth, content, memory);
            memory.doublePremiseTask(content, truth, budget);
        } */
    }

    /// 模拟`CompositionalRules.decomposeStatement`
    ///
    /// # 📄OpenNARS
    ///
    /// {(||, S, P), P} |- S
    /// {(&&, S, P), P} |- S
    ///
    /// @param implication     The implication term to be decomposed
    /// @param componentCommon The part of the implication to be removed
    /// @param compoundTask    Whether the implication comes from the task
    /// @param memory          Reference to the memory
    fn decompose_statement(
        compound: &Term,
        component: &Term,
        compound_task: bool,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        Sentence belief = memory.currentBelief;
        Term content = CompoundTerm.reduceComponents(compound, component, memory);
        if (content == null) {
            return;
        }
        TruthValue truth = null;
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
            memory.doublePremiseTask(content, truth, budget);
            // special inference to answer conjunctive questions with query variables
            if (Variable.containVarQ(sentence.getContent().getName())) {
                Concept contentConcept = memory.termToConcept(content);
                if (contentConcept == null) {
                    return;
                }
                Sentence contentBelief = contentConcept.getBelief(task);
                if (contentBelief == null) {
                    return;
                }
                Task contentTask = new Task(contentBelief, task.getBudget());
                memory.currentTask = contentTask;
                Term conj = Conjunction.make(component, content, memory);
                truth = TruthFunctions.intersection(contentBelief.getTruth(), belief.getTruth());
                budget = BudgetFunctions.compoundForward(truth, conj, memory);
                memory.doublePremiseTask(conj, truth, budget);
            }
        } else {
            TruthValue v1, v2;
            if (compoundTask) {
                v1 = sentence.getTruth();
                v2 = belief.getTruth();
            } else {
                v1 = belief.getTruth();
                v2 = sentence.getTruth();
            }
            if (compound instanceof Conjunction) {
                if (sentence instanceof Sentence) {
                    truth = TruthFunctions.reduceConjunction(v1, v2);
                }
            } else if (compound instanceof Disjunction) {
                if (sentence instanceof Sentence) {
                    truth = TruthFunctions.reduceDisjunction(v1, v2);
                }
            } else {
                return;
            }
            budget = BudgetFunctions.compoundForward(truth, content, memory);
            memory.doublePremiseTask(content, truth, budget);
        } */
    }

    /* --------------- rules used for variable introduction --------------- */

    /// 模拟`CompositionalRules.introVarOuter`
    ///
    /// # 📄OpenNARS
    ///
    /// Introduce a dependent variable in an outer-layer conjunction
    ///
    /// @param taskContent   The first premise <M --> S>
    /// @param beliefContent The second premise <M --> P>
    /// @param index         The location of the shared term: 0 for subject, 1 for predicate
    /// @param memory        Reference to the memory
    fn __intro_var_outer(
        task_statement: &Term,
        belief_statement: &Term,
        index: SyllogismPosition,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        TruthValue truthT = memory.currentTask.getSentence().getTruth();
        TruthValue truthB = memory.currentBelief.getTruth();
        Variable varInd = new Variable("$varInd1");
        Variable varInd2 = new Variable("$varInd2");
        Term term11, term12, term21, term22, commonTerm;
        HashMap<Term, Term> subs = new HashMap<>();
        if (index == 0) {
            term11 = varInd;
            term21 = varInd;
            term12 = taskContent.getPredicate();
            term22 = beliefContent.getPredicate();
            if ((term12 instanceof ImageExt) && (term22 instanceof ImageExt)) {
                commonTerm = ((ImageExt) term12).getTheOtherComponent();
                if ((commonTerm == null) || !((ImageExt) term22).containTerm(commonTerm)) {
                    commonTerm = ((ImageExt) term22).getTheOtherComponent();
                    if ((commonTerm == null) || !((ImageExt) term12).containTerm(commonTerm)) {
                        commonTerm = null;
                    }
                }
                if (commonTerm != null) {
                    subs.put(commonTerm, varInd2);
                    ((ImageExt) term12).applySubstitute(subs);
                    ((ImageExt) term22).applySubstitute(subs);
                }
            }
        } else {
            term11 = taskContent.getSubject();
            term21 = beliefContent.getSubject();
            term12 = varInd;
            term22 = varInd;
            if ((term11 instanceof ImageInt) && (term21 instanceof ImageInt)) {
                commonTerm = ((ImageInt) term11).getTheOtherComponent();
                if ((commonTerm == null) || !((ImageInt) term21).containTerm(commonTerm)) {
                    commonTerm = ((ImageInt) term21).getTheOtherComponent();
                    if ((commonTerm == null) || !((ImageInt) term11).containTerm(commonTerm)) {
                        commonTerm = null;
                    }
                }
                if (commonTerm != null) {
                    subs.put(commonTerm, varInd2);
                    ((ImageInt) term11).applySubstitute(subs);
                    ((ImageInt) term21).applySubstitute(subs);
                }
            }
        }
        Statement state1 = Inheritance.make(term11, term12, memory);
        Statement state2 = Inheritance.make(term21, term22, memory);
        Term content = Implication.make(state1, state2, memory);
        if (content == null) {
            return;
        }
        TruthValue truth = TruthFunctions.induction(truthT, truthB);
        BudgetValue budget = BudgetFunctions.compoundForward(truth, content, memory);
        memory.doublePremiseTask(content, truth, budget);
        content = Implication.make(state2, state1, memory);
        truth = TruthFunctions.induction(truthB, truthT);
        budget = BudgetFunctions.compoundForward(truth, content, memory);
        memory.doublePremiseTask(content, truth, budget);
        content = Equivalence.make(state1, state2, memory);
        truth = TruthFunctions.comparison(truthT, truthB);
        budget = BudgetFunctions.compoundForward(truth, content, memory);
        memory.doublePremiseTask(content, truth, budget);
        Variable varDep = new Variable("#varDep");
        if (index == 0) {
            state1 = Inheritance.make(varDep, taskContent.getPredicate(), memory);
            state2 = Inheritance.make(varDep, beliefContent.getPredicate(), memory);
        } else {
            state1 = Inheritance.make(taskContent.getSubject(), varDep, memory);
            state2 = Inheritance.make(beliefContent.getSubject(), varDep, memory);
        }
        content = Conjunction.make(state1, state2, memory);
        truth = TruthFunctions.intersection(truthT, truthB);
        budget = BudgetFunctions.compoundForward(truth, content, memory);
        memory.doublePremiseTask(content, truth, budget, false); */
    }

    /// 模拟`CompositionalRules.introVarInner`
    ///
    /// # 📄OpenNARS
    ///
    /// {<M --> S>, <C ==> <M --> P>>} |- <(&&, <#x --> S>, C) ==> <#x --> P>>
    /// {<M --> S>, (&&, C, <M --> P>)} |- (&&, C, <<#x --> S> ==> <#x --> P>>)
    ///
    /// @param taskContent   The first premise directly used in internal induction, <M --> S>
    /// @param beliefContent The componentCommon to be used as a premise in internal induction, <M --> P>
    /// @param oldCompound   The whole contentInd of the first premise, Implication or Conjunction
    /// @param memory        Reference to the memory
    fn intro_var_inner(
        premise1_statement: &Term,
        premise2_statement: &Term,
        old_compound: &Term,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence taskSentence = task.getSentence();
        if (!taskSentence.isJudgment() || (premise1.getClass() != premise2.getClass())
                || oldCompound.containComponent(premise1)) {
            return;
        }
        Term subject1 = premise1.getSubject();
        Term subject2 = premise2.getSubject();
        Term predicate1 = premise1.getPredicate();
        Term predicate2 = premise2.getPredicate();
        Term commonTerm1, commonTerm2;
        if (subject1.equals(subject2)) {
            commonTerm1 = subject1;
            commonTerm2 = secondCommonTerm(predicate1, predicate2, 0);
        } else if (predicate1.equals(predicate2)) {
            commonTerm1 = predicate1;
            commonTerm2 = secondCommonTerm(subject1, subject2, 0);
        } else {
            return;
        }
        Sentence belief = memory.currentBelief;
        HashMap<Term, Term> substitute = new HashMap<>();
        substitute.put(commonTerm1, new Variable("#varDep2"));
        CompoundTerm content = (CompoundTerm) Conjunction.make(premise1, oldCompound, memory);
        content.applySubstitute(substitute);
        TruthValue truth = TruthFunctions.intersection(taskSentence.getTruth(), belief.getTruth());
        BudgetValue budget = BudgetFunctions.forward(truth, memory);
        memory.doublePremiseTask(content, truth, budget, false);
        substitute.clear();
        substitute.put(commonTerm1, new Variable("$varInd1"));
        if (commonTerm2 != null) {
            substitute.put(commonTerm2, new Variable("$varInd2"));
        }
        content = Implication.make(premise1, oldCompound, memory);
        if (content == null) {
            return;
        }
        content.applySubstitute(substitute);
        if (premise1.equals(taskSentence.getContent())) {
            truth = TruthFunctions.induction(belief.getTruth(), taskSentence.getTruth());
        } else {
            truth = TruthFunctions.induction(taskSentence.getTruth(), belief.getTruth());
        }
        budget = BudgetFunctions.forward(truth, memory);
        memory.doublePremiseTask(content, truth, budget); */
    }

    /// 模拟`CompositionalRules.secondCommonTerm`
    ///
    /// # 📄OpenNARS
    ///
    /// Introduce a second independent variable into two terms component
    /// @param term1 The first term
    /// @param term2 The second term
    /// @param index The index of the terms in their statement
    fn __second_common_term<'a>(
        term1: &'a Term,
        term2: &'a Term,
        index: SyllogismPosition,
    ) -> &'a Term {
        /* 📄OpenNARS源码：
        Term commonTerm = null;
        if (index == 0) {
            if ((term1 instanceof ImageExt) && (term2 instanceof ImageExt)) {
                commonTerm = ((ImageExt) term1).getTheOtherComponent();
                if ((commonTerm == null) || !((ImageExt) term2).containTerm(commonTerm)) {
                    commonTerm = ((ImageExt) term2).getTheOtherComponent();
                    if ((commonTerm == null) || !((ImageExt) term1).containTerm(commonTerm)) {
                        commonTerm = null;
                    }
                }
            }
        } else {
            if ((term1 instanceof ImageInt) && (term2 instanceof ImageInt)) {
                commonTerm = ((ImageInt) term1).getTheOtherComponent();
                if ((commonTerm == null) || !((ImageInt) term2).containTerm(commonTerm)) {
                    commonTerm = ((ImageInt) term2).getTheOtherComponent();
                    if ((commonTerm == null) || !((ImageExt) term1).containTerm(commonTerm)) {
                        commonTerm = null;
                    }
                }
            }
        }
        return commonTerm; */
        todo!("// TODO: 待实现")
    }
}

/// 自动实现，以便添加方法
impl<C: ReasonContext, T: DerivationContext<C>> CompositionalRules<C> for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
