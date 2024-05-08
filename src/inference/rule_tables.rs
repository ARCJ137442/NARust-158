//! 🎯复刻OpenNARS `nars.inference.RuleTables`
//! * ❓【2024-05-06 21:59:43】是否一定要按照OpenNARS中「全静态方法类」来实现
//!   * 🚩🆕【2024-05-06 21:59:59】目前决定：用更Rusty的方式——模块内全局函数
//!   * 🚩【2024-05-06 22:28:30】最新方法：上下文+特征追加
//!     * 📄参见[「推理上下文」](super::reason_context)
//! * ✅基本完成「特征方法」API：函数签名、返回值、参数类型

use super::ReasonContext;
use crate::{entity::*, language::*, storage::*};

/// 🆕用于表征[`RuleTables::index_to_figure`]推导出的「三段论子类型」
/// * 📝OpenNARS中是在「三段论推理」的「陈述🆚陈述」中表示「位置关系」
///   * 📄`<A --> B>`与`<B --> C>`中，`B`就分别在`1`、`0`两个索引位置
///     * 📌因此有`SP`或`Subject-Predicate`
///     * 📌同时也有了其它三种「陈述图式」
///
/// # 📄OpenNARS
///
/// location of the shared term
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyllogismFigure {
    /// 主项对主项
    SubjectSubject,

    /// 主项对谓项
    SubjectPredicate,

    /// 谓项对主项
    PredicateSubject,

    /// 谓项对谓项
    PredicatePredicate,
}
use SyllogismFigure::*;

/// 模拟`RuleTables`
/// * 🚩【2024-05-07 01:56:57】现在通过「推理上下文」自动锁定其内的「子类型」
pub trait RuleTables: ReasonContext {
    /// 模拟`RuleTables.reason`
    /// * 🚩【2024-05-08 16:36:34】仅保留「记忆区」单个参数
    ///   * 📌情况：该函数只在[`Memory::__fire_concept`]调用，且其中的`task_link`也固定为「当前任务链」
    ///   * 📌原因：同时传入「自身可变引用」与「自身不可变引用」⇒借用错误
    ///
    /// TODO: 🏗️【2024-05-08 17:04:04】后续要简化这类耦合情形
    ///
    /// # 📄OpenNARS
    ///
    /// Entry point of the inference engine
    ///
    /// @param tLink  The selected TaskLink, which will provide a task
    /// @param bLink  The selected TermLink, which may provide a belief
    /// @param memory Reference to the memory
    fn reason(
        /* task_link: &Self::TermLink, term_link: &Self::TaskLink,  */
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS：
        Task task = memory.currentTask;
        Sentence taskSentence = task.getSentence();
        Term taskTerm = (Term) taskSentence.getContent().clone(); // cloning for substitution
        Term beliefTerm = (Term) bLink.getTarget().clone(); // cloning for substitution
        Concept beliefConcept = memory.termToConcept(beliefTerm);
        Sentence belief = null;
        if (beliefConcept != null) {
            belief = beliefConcept.getBelief(task);
        }
        memory.currentBelief = belief; // may be null
        if (belief != null) {
            LocalRules.match(task, belief, memory);
        }
        if (!memory.noResult() && task.getSentence().isJudgment()) {
            return;
        }
        short tIndex = tLink.getIndex(0);
        short bIndex = bLink.getIndex(0);
        switch (tLink.getType()) { // dispatch first by TaskLink type
            case TermLink.SELF:
                switch (bLink.getType()) {
                    case TermLink.COMPONENT:
                        compoundAndSelf((CompoundTerm) taskTerm, beliefTerm, true, memory);
                        break;
                    case TermLink.COMPOUND:
                        compoundAndSelf((CompoundTerm) beliefTerm, taskTerm, false, memory);
                        break;
                    case TermLink.COMPONENT_STATEMENT:
                        if (belief != null) {
                            SyllogisticRules.detachment(task.getSentence(), belief, bIndex, memory);
                        }
                        break;
                    case TermLink.COMPOUND_STATEMENT:
                        if (belief != null) {
                            SyllogisticRules.detachment(belief, task.getSentence(), bIndex, memory);
                        }
                        break;
                    case TermLink.COMPONENT_CONDITION:
                        if (belief != null) {
                            bIndex = bLink.getIndex(1);
                            SyllogisticRules.conditionalDedInd((Implication) taskTerm, bIndex, beliefTerm, tIndex,
                                    memory);
                        }
                        break;
                    case TermLink.COMPOUND_CONDITION:
                        if (belief != null) {
                            bIndex = bLink.getIndex(1);
                            SyllogisticRules.conditionalDedInd((Implication) beliefTerm, bIndex, taskTerm, tIndex,
                                    memory);
                        }
                        break;
                }
                break;
            case TermLink.COMPOUND:
                switch (bLink.getType()) {
                    case TermLink.COMPOUND:
                        compoundAndCompound((CompoundTerm) taskTerm, (CompoundTerm) beliefTerm, memory);
                        break;
                    case TermLink.COMPOUND_STATEMENT:
                        compoundAndStatement((CompoundTerm) taskTerm, tIndex, (Statement) beliefTerm, bIndex,
                                beliefTerm, memory);
                        break;
                    case TermLink.COMPOUND_CONDITION:
                        if (belief != null) {
                            if (beliefTerm instanceof Implication) {
                                if (Variable.unify(Symbols.VAR_INDEPENDENT, ((Implication) beliefTerm).getSubject(),
                                        taskTerm, beliefTerm, taskTerm)) {
                                    detachmentWithVar(belief, taskSentence, bIndex, memory);
                                } else {
                                    SyllogisticRules.conditionalDedInd((Implication) beliefTerm, bIndex, taskTerm, -1,
                                            memory);
                                }
                            } else if (beliefTerm instanceof Equivalence) {
                                SyllogisticRules.conditionalAna((Equivalence) beliefTerm, bIndex, taskTerm, -1, memory);
                            }
                        }
                        break;
                }
                break;
            case TermLink.COMPOUND_STATEMENT:
                switch (bLink.getType()) {
                    case TermLink.COMPONENT:
                        componentAndStatement((CompoundTerm) memory.currentTerm, bIndex, (Statement) taskTerm, tIndex,
                                memory);
                        break;
                    case TermLink.COMPOUND:
                        compoundAndStatement((CompoundTerm) beliefTerm, bIndex, (Statement) taskTerm, tIndex,
                                beliefTerm, memory);
                        break;
                    case TermLink.COMPOUND_STATEMENT:
                        if (belief != null) {
                            syllogisms(tLink, bLink, taskTerm, beliefTerm, memory);
                        }
                        break;
                    case TermLink.COMPOUND_CONDITION:
                        if (belief != null) {
                            bIndex = bLink.getIndex(1);
                            if (beliefTerm instanceof Implication) {
                                conditionalDedIndWithVar((Implication) beliefTerm, bIndex, (Statement) taskTerm, tIndex,
                                        memory);
                            }
                        }
                        break;
                }
                break;
            case TermLink.COMPOUND_CONDITION:
                switch (bLink.getType()) {
                    case TermLink.COMPOUND:
                        if (belief != null) {
                            detachmentWithVar(taskSentence, belief, tIndex, memory);
                        }
                        break;
                    case TermLink.COMPOUND_STATEMENT:
                        if (belief != null) {
                            if (taskTerm instanceof Implication) // TODO maybe put instanceof test within
                                                                 // conditionalDedIndWithVar()
                            {
                                Term subj = ((Implication) taskTerm).getSubject();
                                if (subj instanceof Negation) {
                                    if (task.getSentence().isJudgment()) {
                                        componentAndStatement((CompoundTerm) subj, bIndex, (Statement) taskTerm, tIndex,
                                                memory);
                                    } else {
                                        componentAndStatement((CompoundTerm) subj, tIndex, (Statement) beliefTerm,
                                                bIndex, memory);
                                    }
                                } else {
                                    conditionalDedIndWithVar((Implication) taskTerm, tIndex, (Statement) beliefTerm,
                                            bIndex, memory);
                                }
                            }
                            break;
                        }
                        break;
                }
        } */
        let task_link = memory.current_task_link();
        let term_link = memory
            .current_belief_link()
            .as_ref()
            .expect("此处必须有：在调用前设定了非空值");
        task_link;
        term_link;
        todo!("// TODO: 有待实现")
    }

    /* ----- syllogistic inferences ----- */

    /// 模拟`RuleTables.syllogisms`
    ///
    /// # 📄OpenNARS
    ///
    /// Meta-table of syllogistic rules, indexed by the content classes of the taskSentence and the belief
    ///
    /// @param tLink      The link to task
    /// @param bLink      The link to belief
    /// @param taskTerm   The content of task
    /// @param beliefTerm The content of belief
    /// @param memory     Reference to the memory
    fn __syllogisms(task_link: &Self::TermLink, term_link: &Self::TermLink) {
        /* 📄OpenNARS源码：
        Sentence taskSentence = memory.currentTask.getSentence();
        Sentence belief = memory.currentBelief;
        int figure;
        if (taskTerm instanceof Inheritance) {
            if (beliefTerm instanceof Inheritance) {
                figure = indexToFigure(tLink, bLink);
                asymmetricAsymmetric(taskSentence, belief, figure, memory);
            } else if (beliefTerm instanceof Similarity) {
                figure = indexToFigure(tLink, bLink);
                asymmetricSymmetric(taskSentence, belief, figure, memory);
            } else {
                detachmentWithVar(belief, taskSentence, bLink.getIndex(0), memory);
            }
        } else if (taskTerm instanceof Similarity) {
            if (beliefTerm instanceof Inheritance) {
                figure = indexToFigure(bLink, tLink);
                asymmetricSymmetric(belief, taskSentence, figure, memory);
            } else if (beliefTerm instanceof Similarity) {
                figure = indexToFigure(bLink, tLink);
                symmetricSymmetric(belief, taskSentence, figure, memory);
            }
        } else if (taskTerm instanceof Implication) {
            if (beliefTerm instanceof Implication) {
                figure = indexToFigure(tLink, bLink);
                asymmetricAsymmetric(taskSentence, belief, figure, memory);
            } else if (beliefTerm instanceof Equivalence) {
                figure = indexToFigure(tLink, bLink);
                asymmetricSymmetric(taskSentence, belief, figure, memory);
            } else if (beliefTerm instanceof Inheritance) {
                detachmentWithVar(taskSentence, belief, tLink.getIndex(0), memory);
            }
        } else if (taskTerm instanceof Equivalence) {
            if (beliefTerm instanceof Implication) {
                figure = indexToFigure(bLink, tLink);
                asymmetricSymmetric(belief, taskSentence, figure, memory);
            } else if (beliefTerm instanceof Equivalence) {
                figure = indexToFigure(bLink, tLink);
                symmetricSymmetric(belief, taskSentence, figure, memory);
            } else if (beliefTerm instanceof Inheritance) {
                detachmentWithVar(taskSentence, belief, tLink.getIndex(0), memory);
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.indexToFigure`
    ///
    /// # 📄OpenNARS
    ///
    /// Decide the figure of syllogism according to the locations of the common term in the premises
    ///
    /// @param link1 The link to the first premise
    /// @param link2 The link to the second premise
    /// @return The figure of the syllogism, one of the four: 11, 12, 21, or 22
    fn __index_to_figure(
        term_link_1: &Self::TermLink,
        term_link_2: &Self::TermLink,
    ) -> SyllogismFigure {
        /* 📄OpenNARS源码：
        return (link1.getIndex(0) + 1) * 10 + (link2.getIndex(0) + 1); */
        // ? 【2024-05-06 22:58:22】是否要假定？是否应该在调用处将「首个索引」传入？
        debug_assert!(term_link_1.type_ref().has_indexes());
        debug_assert!(term_link_2.type_ref().has_indexes());
        let root_index_1 = term_link_1.get_index(0).unwrap();
        let root_index_2 = term_link_2.get_index(0).unwrap();
        // * 🚩核心：0→主项，1→谓项，整体`<主项 --> 谓项>`
        match (root_index_1, root_index_2) {
            // 四个位置
            (0, 0) => SubjectSubject,
            (0, 1) => SubjectPredicate,
            (1, 0) => PredicateSubject,
            (1, 1) => PredicatePredicate,
            // 不可达的情况
            _ => unreachable!("不应该出现的位置：({root_index_1}, {root_index_2})"),
        }
    }

    /// 模拟`RuleTables.asymmetricAsymmetric`
    ///
    /// # 📄OpenNARS
    ///
    /// Syllogistic rules whose both premises are on the same asymmetric relation
    ///
    /// @param sentence The taskSentence in the task
    /// @param belief   The judgment in the belief
    /// @param figure   The location of the shared term
    /// @param memory   Reference to the memory
    fn __asymmetric_asymmetric(
        sentence: &Self::Sentence,
        belief: &Self::Sentence,
        figure: SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Statement s1 = (Statement) sentence.cloneContent();
        Statement s2 = (Statement) belief.cloneContent();
        Term t1, t2;
        switch (figure) {
            case 11: // induction
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getSubject(), s2.getSubject(), s1, s2)) {
                    if (s1.equals(s2)) {
                        return;
                    }
                    t1 = s2.getPredicate();
                    t2 = s1.getPredicate();
                    CompositionalRules.composeCompound(s1, s2, 0, memory);
                    SyllogisticRules.abdIndCom(t1, t2, sentence, belief, figure, memory);
                }

                break;
            case 12: // deduction
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getSubject(), s2.getPredicate(), s1, s2)) {
                    if (s1.equals(s2)) {
                        return;
                    }
                    t1 = s2.getSubject();
                    t2 = s1.getPredicate();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, s1, s2)) {
                        LocalRules.matchReverse(memory);
                    } else {
                        SyllogisticRules.dedExe(t1, t2, sentence, belief, memory);
                    }
                }
                break;
            case 21: // exemplification
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getPredicate(), s2.getSubject(), s1, s2)) {
                    if (s1.equals(s2)) {
                        return;
                    }
                    t1 = s1.getSubject();
                    t2 = s2.getPredicate();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, s1, s2)) {
                        LocalRules.matchReverse(memory);
                    } else {
                        SyllogisticRules.dedExe(t1, t2, sentence, belief, memory);
                    }
                }
                break;
            case 22: // abduction
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getPredicate(), s2.getPredicate(), s1, s2)) {
                    if (s1.equals(s2)) {
                        return;
                    }
                    t1 = s1.getSubject();
                    t2 = s2.getSubject();
                    if (!SyllogisticRules.conditionalAbd(t1, t2, s1, s2, memory)) { // if conditional abduction, skip
                                                                                    // the following
                        CompositionalRules.composeCompound(s1, s2, 1, memory);
                        SyllogisticRules.abdIndCom(t1, t2, sentence, belief, figure, memory);
                    }
                }
                break;
            default:
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.asymmetricSymmetric`
    ///
    /// # 📄OpenNARS
    ///
    /// Syllogistic rules whose first premise is on an asymmetric relation, and the second on a symmetric relation
    ///
    /// @param asym   The asymmetric premise
    /// @param sym    The symmetric premise
    /// @param figure The location of the shared term
    /// @param memory Reference to the memory
    fn __asymmetric_symmetric(
        asymmetric: &Self::Sentence,
        symmetric: &Self::Sentence,
        figure: SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Statement asymSt = (Statement) asym.cloneContent();
        Statement symSt = (Statement) sym.cloneContent();
        Term t1, t2;
        switch (figure) {
            case 11:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, asymSt.getSubject(), symSt.getSubject(), asymSt, symSt)) {
                    t1 = asymSt.getPredicate();
                    t2 = symSt.getPredicate();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, asymSt, symSt)) {
                        LocalRules.matchAsymSym(asym, sym, figure, memory);
                    } else {
                        SyllogisticRules.analogy(t2, t1, asym, sym, figure, memory);
                    }
                }
                break;
            case 12:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, asymSt.getSubject(), symSt.getPredicate(), asymSt, symSt)) {
                    t1 = asymSt.getPredicate();
                    t2 = symSt.getSubject();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, asymSt, symSt)) {
                        LocalRules.matchAsymSym(asym, sym, figure, memory);
                    } else {
                        SyllogisticRules.analogy(t2, t1, asym, sym, figure, memory);
                    }
                }
                break;
            case 21:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, asymSt.getPredicate(), symSt.getSubject(), asymSt, symSt)) {
                    t1 = asymSt.getSubject();
                    t2 = symSt.getPredicate();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, asymSt, symSt)) {
                        LocalRules.matchAsymSym(asym, sym, figure, memory);
                    } else {
                        SyllogisticRules.analogy(t1, t2, asym, sym, figure, memory);
                    }
                }
                break;
            case 22:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, asymSt.getPredicate(), symSt.getPredicate(), asymSt,
                        symSt)) {
                    t1 = asymSt.getSubject();
                    t2 = symSt.getSubject();
                    if (Variable.unify(Symbols.VAR_QUERY, t1, t2, asymSt, symSt)) {
                        LocalRules.matchAsymSym(asym, sym, figure, memory);
                    } else {
                        SyllogisticRules.analogy(t1, t2, asym, sym, figure, memory);
                    }
                }
                break;
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.symmetricSymmetric`
    ///
    /// # 📄OpenNARS
    ///
    /// Syllogistic rules whose both premises are on the same symmetric relation
    ///
    /// @param belief       The premise that comes from a belief
    /// @param taskSentence The premise that comes from a task
    /// @param figure       The location of the shared term
    /// @param memory       Reference to the memory
    fn __symmetric_symmetric(
        belief: &Self::Sentence,
        task_sentence: &Self::Sentence,
        figure: SyllogismFigure,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Statement s1 = (Statement) belief.cloneContent();
        Statement s2 = (Statement) taskSentence.cloneContent();
        switch (figure) {
            case 11:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getSubject(), s2.getSubject(), s1, s2)) {
                    SyllogisticRules.resemblance(s1.getPredicate(), s2.getPredicate(), belief, taskSentence, figure,
                            memory);
                }
                break;
            case 12:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getSubject(), s2.getPredicate(), s1, s2)) {
                    SyllogisticRules.resemblance(s1.getPredicate(), s2.getSubject(), belief, taskSentence, figure,
                            memory);
                }
                break;
            case 21:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getPredicate(), s2.getSubject(), s1, s2)) {
                    SyllogisticRules.resemblance(s1.getSubject(), s2.getPredicate(), belief, taskSentence, figure,
                            memory);
                }
                break;
            case 22:
                if (Variable.unify(Symbols.VAR_INDEPENDENT, s1.getPredicate(), s2.getPredicate(), s1, s2)) {
                    SyllogisticRules.resemblance(s1.getSubject(), s2.getSubject(), belief, taskSentence, figure,
                            memory);
                }
                break;
        } */
        todo!("// TODO: 有待实现")
    }

    /* ----- conditional inferences ----- */

    /// 模拟`RuleTables.detachmentWithVar`
    ///
    /// # 📄OpenNARS
    ///
    /// The detachment rule, with variable unification
    ///
    /// @param originalMainSentence The premise that is an Implication or Equivalence
    /// @param subSentence          The premise that is the subject or predicate of the first one
    /// @param index                The location of the second premise in the first
    /// @param memory               Reference to the memory
    fn __detachment_with_var(
        original_main_sentence: &Self::Sentence,
        sub_sentence: &Self::Sentence,
        index: usize,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        Sentence mainSentence = (Sentence) originalMainSentence.clone(); // for substitution
        Statement statement = (Statement) mainSentence.getContent();
        Term component = statement.componentAt(index);
        Term content = subSentence.getContent();
        if (((component instanceof Inheritance) || (component instanceof Negation)) && (memory.currentBelief != null)) {
            if (component.isConstant()) {
                SyllogisticRules.detachment(mainSentence, subSentence, index, memory);
            } else if (Variable.unify(Symbols.VAR_INDEPENDENT, component, content, statement, content)) {
                SyllogisticRules.detachment(mainSentence, subSentence, index, memory);
            } else if ((statement instanceof Implication) && (statement.getPredicate() instanceof Statement)
                    && (memory.currentTask.getSentence().isJudgment())) {
                Statement s2 = (Statement) statement.getPredicate();
                if (s2.getSubject().equals(((Statement) content).getSubject())) {
                    CompositionalRules.introVarInner((Statement) content, s2, statement, memory);
                }
                CompositionalRules.IntroVarSameSubjectOrPredicate(originalMainSentence, subSentence, component, content,
                        index, memory);
            } else if ((statement instanceof Equivalence) && (statement.getPredicate() instanceof Statement)
                    && (memory.currentTask.getSentence().isJudgment())) {
                CompositionalRules.IntroVarSameSubjectOrPredicate(originalMainSentence, subSentence, component, content,
                        index, memory);
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.conditionalDedIndWithVar`
    /// * ❌【2024-05-06 23:27:58】这里没法对「条件词项」「陈述」做更多限制
    ///   * 需要断言「条件词项」`conditional`必须是「蕴含」
    ///
    /// # 📄OpenNARS
    ///
    /// Conditional deduction or induction, with variable unification
    ///
    /// @param conditional The premise that is an Implication with a Conjunction as condition
    /// @param index       The location of the shared term in the condition
    /// @param statement   The second premise that is a statement
    /// @param side        The location of the shared term in the statement
    /// @param memory      Reference to the memory
    fn __conditional_ded_ind_with_var(
        conditional: &Term,
        index: usize,
        statement: &Term,
        side: usize,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        CompoundTerm condition = (CompoundTerm) conditional.getSubject();
        Term component = condition.componentAt(index);
        Term component2 = null;
        if (statement instanceof Inheritance) {
            component2 = statement;
            side = -1;
        } else if (statement instanceof Implication) {
            component2 = statement.componentAt(side);
        }
        if (component2 != null) {
            boolean unifiable = Variable.unify(Symbols.VAR_INDEPENDENT, component, component2, conditional, statement);
            if (!unifiable) {
                unifiable = Variable.unify(Symbols.VAR_DEPENDENT, component, component2, conditional, statement);
            }
            if (unifiable) {
                SyllogisticRules.conditionalDedInd(conditional, index, statement, side, memory);
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /* ----- structural inferences ----- */

    /// 模拟`RuleTables.compoundAndSelf`
    ///
    /// # 📄OpenNARS
    ///
    /// Inference between a compound term and a component of it
    ///
    /// @param compound     The compound term
    /// @param component    The component term
    /// @param compoundTask Whether the compound comes from the task
    /// @param memory       Reference to the memory
    fn __compound_and_self(
        compound: &Term,
        componment: &Term,
        compound_task: bool,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        if ((compound instanceof Conjunction) || (compound instanceof Disjunction)) {
            if (memory.currentBelief != null) {
                CompositionalRules.decomposeStatement(compound, component, compoundTask, memory);
            } else if (compound.containComponent(component)) {
                StructuralRules.structuralCompound(compound, component, compoundTask, memory);
            }
            // } else if ((compound instanceof Negation) &&
            // !memory.currentTask.isStructural()) {
        } else if (compound instanceof Negation) {
            if (compoundTask) {
                StructuralRules.transformNegation(((Negation) compound).componentAt(0), memory);
            } else {
                StructuralRules.transformNegation(compound, memory);
            } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.compoundAndCompound`
    ///
    /// # 📄OpenNARS
    ///
    /// Inference between two compound terms
    ///
    /// @param taskTerm   The compound from the task
    /// @param beliefTerm The compound from the belief
    /// @param memory     Reference to the memory
    fn __compound_and_compound(task_term: &Term, belief_term: &Term, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        if (taskTerm.getClass() == beliefTerm.getClass()) {
            if (taskTerm.size() > beliefTerm.size()) {
                compoundAndSelf(taskTerm, beliefTerm, true, memory);
            } else if (taskTerm.size() < beliefTerm.size()) {
                compoundAndSelf(beliefTerm, taskTerm, false, memory);
            }
        } */
        todo!("// TODO: 有待实现")
    }

    /// 模拟`RuleTables.compoundAndStatement`
    ///
    /// # 📄OpenNARS
    ///
    /// Inference between a compound term and a statement
    ///
    /// @param compound   The compound term
    /// @param index      The location of the current term in the compound
    /// @param statement  The statement
    /// @param side       The location of the current term in the statement
    /// @param beliefTerm The content of the belief
    /// @param memory     Reference to the memory
    fn __compound_and_statement(
        compound: &Term,
        index: usize,
        statement: &Term,
        side: usize,
        belief_term: &Term,
        memory: &mut Self::Memory,
    ) {
        /* 📄OpenNARS源码：
        // if (!memory.currentTask.isStructural()) {
        if (statement instanceof Inheritance) {
            StructuralRules.structuralDecompose1(compound, index, statement, memory);
            if (!(compound instanceof SetExt) && !(compound instanceof SetInt)) {
                StructuralRules.structuralDecompose2(statement, index, memory); // {(C-B) --> (C-A), A @ (C-A)} |- A -->
                                                                                // B
            } else {
                StructuralRules.transformSetRelation(compound, statement, side, memory);
            }
        } else if (statement instanceof Similarity) {
            StructuralRules.structuralDecompose2(statement, index, memory); // {(C-B) --> (C-A), A @ (C-A)} |- A --> B
            if ((compound instanceof SetExt) || (compound instanceof SetInt)) {
                StructuralRules.transformSetRelation(compound, statement, side, memory);
            }
        } else if ((statement instanceof Implication) && (compound instanceof Negation)) {
            if (index == 0) {
                StructuralRules.contraposition(statement, memory.currentTask.getSentence(), memory);
            } else {
                StructuralRules.contraposition(statement, memory.currentBelief, memory);
            }
        }
        // } */
        todo!("// TODO: 有待实现")
    }

    /* ----- inference with one TaskLink only ----- */

    /// 模拟`RuleTables.transformTask`
    /// * 🚩【2024-05-08 16:36:34】仅保留「记忆区」单个参数
    ///   * 📌情况：该函数只在[`Memory::__fire_concept`]调用，且其中的`task_link`也固定为「当前任务链」
    ///   * 📌原因：同时传入「自身可变引用」与「自身不可变引用」⇒借用错误
    ///
    /// # 📄OpenNARS
    ///
    /// The TaskLink is of type TRANSFORM, and the conclusion is an equivalent
    /// transformation
    ///
    /// @param tLink  The task link
    /// @param memory Reference to the memory
    fn transform_task(/* task_link: &Self::TaskLink,  */ memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        CompoundTerm content = (CompoundTerm) memory.currentTask.getContent().clone();
        short[] indices = tLink.getIndices();
        Term inh = null;
        if ((indices.length == 2) || (content instanceof Inheritance)) { // <(*, term, #) --> #>
            inh = content;
        } else if (indices.length == 3) { // <<(*, term, #) --> #> ==> #>
            inh = content.componentAt(indices[0]);
        } else if (indices.length == 4) { // <(&&, <(*, term, #) --> #>, #) ==> #>
            Term component = content.componentAt(indices[0]);
            if ((component instanceof Conjunction)
                    && (((content instanceof Implication) && (indices[0] == 0)) || (content instanceof Equivalence))) {
                inh = ((CompoundTerm) component).componentAt(indices[1]);
            } else {
                return;
            }
        }
        if (inh instanceof Inheritance) {
            StructuralRules.transformProductImage((Inheritance) inh, content, indices, memory);
        } */
        let task_link = memory.current_task_link();
        todo!("// TODO: 有待实现")
    }
}

/// 自动实现，以便添加方法
impl<T: ReasonContext> RuleTables for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
