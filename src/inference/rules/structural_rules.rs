//! 🎯复刻OpenNARS `nars.inference.StructuralRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::type_context)
//!
//! * ✅【2024-05-11 15:10:00】初步复现方法API
//!
//! TODO: 🚧完成具体实现

use crate::{
    control::*,
    entity::*,
    global::Float,
    inference::*,
    language::{CompoundTermRef, Term},
    nars::DEFAULT_PARAMETERS,
    types::TypeContext,
};

/// 模拟`StructuralRules`
/// * 📝这些规则均是有关「复合词项」的规则
///   * 📄诸如「NAL-3集合规则」「NAL-4关系规则」「NAL-5命题规则」等
///   * ❓似乎未涉及到NAL-6
/// * 📝【2024-05-11 15:03:22】OpenNARS中使用`memory`记忆区引用的地方，往往都是最后「递交推理结果」的`singlePremiseTask`等方法
///   * 💭这些完全可以延后，比如放到专门的「推理上下文」中
///
/// # 📄OpenNARS
///
/// Single-premise inference rules involving compound terms. Input are one sentence (the premise) and one TermLink (indicating a component)
pub trait StructuralRules<C: TypeContext> {
    /// 模拟`StructuralRules.RELIANCE`
    const __RELIANCE: Float = DEFAULT_PARAMETERS.reliance;

    /* -------------------- transform between compounds and components -------------------- */

    /// 模拟`StructuralRules.structuralCompose2`
    /// * 📝外延差、外延交的分配律——外延交差分配律
    /// * 📌【2024-05-11 14:21:20】目前认为`side`只有「主项/谓项」两种
    ///   * 🚩使用[`SyllogismPosition`]
    /// * ❓函数名末尾的「2」是何含义？
    ///
    /// # 📄OpenNARS
    ///
    /// {<S --> P>, S@(S&T)} |- <(S&T) --> (P&T)>
    /// {<S --> P>, S@(M-S)} |- <(M-P) --> (M-S)>
    ///
    /// @param compound  The compound term
    /// @param index     The location of the indicated term in the compound
    /// @param statement The premise
    /// @param side      The location of the indicated term in the premise
    /// @param memory    Reference to the memory
    fn structural_compose2(
        compound: &Term,
        index: usize,
        statement: &Term,
        side: SyllogismPosition,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (compound.equals(statement.componentAt(side))) {
            return;
        }
        Term sub = statement.getSubject();
        Term pred = statement.getPredicate();
        ArrayList<Term> components = compound.cloneComponents();
        if (((side == 0) && components.contains(pred)) || ((side == 1) && components.contains(sub))) {
            return;
        }
        if (side == 0) {
            if (components.contains(sub)) {
                sub = compound;
                components.set(index, pred);
                pred = CompoundTerm.make(compound, components, memory);
            }
        } else {
            if (components.contains(pred)) {
                components.set(index, sub);
                sub = CompoundTerm.make(compound, components, memory);
                pred = compound;
            }
        }
        if ((sub == null) || (pred == null)) {
            return;
        }
        Term content;
        if (switchOrder(compound, index)) {
            content = Statement.make(statement, pred, sub, memory);
        } else {
            content = Statement.make(statement, sub, pred, memory);
        }
        if (content == null) {
            return;
        }
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackwardWeak(content, memory);
        } else {
            if (compound.size() > 1) {
                if (sentence.isJudgment()) {
                    truth = TruthFunctions.deduction(truth, RELIANCE);
                } else {
                    return;
                }
            }
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    /// 模拟`StructuralRules.structuralDecompose2`
    /// * 📝结构消去律
    /// * ❓函数名末尾的「2」是何含义？
    ///
    /// # 📄OpenNARS
    ///
    /// {<(S&T) --> (P&T)>, S@(S&T)} |- <S --> P>
    ///
    /// @param statement The premise
    /// @param memory    Reference to the memory
    fn structural_decompose2(statement: &Term, memory: &mut C::Memory) {
        /* 📄OpenNARS源码：
        Term subj = statement.getSubject();
        Term pred = statement.getPredicate();
        if (subj.getClass() != pred.getClass()) {
            return;
        }
        CompoundTerm sub = (CompoundTerm) subj;
        CompoundTerm pre = (CompoundTerm) pred;
        if (sub.size() != pre.size() || sub.size() <= index) {
            return;
        }
        Term t1 = sub.componentAt(index);
        Term t2 = pre.componentAt(index);
        Term content;
        if (switchOrder(sub, (short) index)) {
            content = Statement.make(statement, t2, t1, memory);
        } else {
            content = Statement.make(statement, t1, t2, memory);
        }
        if (content == null) {
            return;
        }
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
        } else {
            if (!(sub instanceof Product) && (sub.size() > 1) && (sentence.isJudgment())) {
                return;
            }
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    /// 模拟`StructuralRules.switchOrder`
    /// * 📝【2024-05-11 14:38:06】一个判别函数，但作用尚未完全清楚
    /// * 🚩🆕根据实际逻辑做了简化：「词项差 && 索引==1」||「像 && 索引不等于」
    ///   * 💭其中有关「像」的逻辑，目前理解是「旨在不关注占位符」
    ///
    /// # 📄OpenNARS
    ///
    /// List the cases where the direction of inheritance is revised in conclusion
    ///
    /// @param compound The compound term
    /// @param index    The location of focus in the compound
    /// @return Whether the direction of inheritance should be revised
    fn __switch_order(compound: CompoundTermRef, index: usize) -> bool {
        /* 📄OpenNARS源码：
        return ((((compound instanceof DifferenceExt) || (compound instanceof DifferenceInt)) && (index == 1))
                || ((compound instanceof ImageExt) && (index != ((ImageExt) compound).getRelationIndex()))
                || ((compound instanceof ImageInt) && (index != ((ImageInt) compound).getRelationIndex()))); */
        compound.inner.instanceof_difference() && (index == 1)
            || compound.inner.instanceof_image() && index != compound.get_placeholder_index()
    }

    /// 模拟`StructuralRules.structuralCompose1`
    /// * 📝单个位置的替换
    /// * ⚠️此处的`index`是指「在复合词项中的索引」
    ///
    /// # 📄OpenNARS
    ///
    /// {<S --> P>, P@(P&Q)} |- <S --> (P&Q)>
    ///
    /// @param compound  The compound term
    /// @param index     The location of the indicated term in the compound
    /// @param statement The premise
    /// @param memory    Reference to the memory
    fn structural_compose1(
        compound: &Term,
        index: usize,
        statement: &Term,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (!memory.currentTask.getSentence().isJudgment()) {
            return;
        }
        Term component = compound.componentAt(index);
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        TruthValue truthDed = TruthFunctions.deduction(truth, RELIANCE);
        TruthValue truthNDed = TruthFunctions.negation(TruthFunctions.deduction(truth, RELIANCE));
        Term subj = statement.getSubject();
        Term pred = statement.getPredicate();
        if (component.equals(subj)) {
            if (compound instanceof IntersectionExt) {
                structuralStatement(compound, pred, truthDed, memory);
            } else if (compound instanceof IntersectionInt) {
            } else if ((compound instanceof DifferenceExt) && (index == 0)) {
                structuralStatement(compound, pred, truthDed, memory);
            } else if (compound instanceof DifferenceInt) {
                if (index == 0) {
                } else {
                    structuralStatement(compound, pred, truthNDed, memory);
                }
            }
        } else if (component.equals(pred)) {
            if (compound instanceof IntersectionExt) {
            } else if (compound instanceof IntersectionInt) {
                structuralStatement(subj, compound, truthDed, memory);
            } else if (compound instanceof DifferenceExt) {
                if (index == 0) {
                } else {
                    structuralStatement(subj, compound, truthNDed, memory);
                }
            } else if ((compound instanceof DifferenceInt) && (index == 0)) {
                structuralStatement(subj, compound, truthDed, memory);
            }
        } */
    }

    /// 模拟`StructuralRules.structuralDecompose1`
    /// * 📝单词项解构
    ///
    /// # 📄OpenNARS
    ///
    /// {<(S&T) --> P>, S@(S&T)} |- <S --> P>
    ///
    /// @param compound  The compound term
    /// @param index     The location of the indicated term in the compound
    /// @param statement The premise
    /// @param memory    Reference to the memory
    fn structural_decompose1(
        compound: &Term,
        index: usize,
        statement: &Term,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (!memory.currentTask.getSentence().isJudgment()) {
            return;
        }
        Term component = compound.componentAt(index);
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        TruthValue truthDed = TruthFunctions.deduction(truth, RELIANCE);
        TruthValue truthNDed = TruthFunctions.negation(TruthFunctions.deduction(truth, RELIANCE));
        Term subj = statement.getSubject();
        Term pred = statement.getPredicate();
        if (compound.equals(subj)) {
            if (compound instanceof IntersectionInt) {
                structuralStatement(component, pred, truthDed, memory);
            } else if ((compound instanceof SetExt) && (compound.size() > 1)) {
                structuralStatement(SetExt.make(component, memory), pred, truthDed, memory);
            } else if (compound instanceof DifferenceInt) {
                if (index == 0) {
                    structuralStatement(component, pred, truthDed, memory);
                } else {
                    structuralStatement(component, pred, truthNDed, memory);
                }
            }
        } else if (compound.equals(pred)) {
            if (compound instanceof IntersectionExt) {
                structuralStatement(subj, component, truthDed, memory);
            } else if ((compound instanceof SetInt) && (compound.size() > 1)) {
                structuralStatement(subj, SetInt.make(component, memory), truthDed, memory);
            } else if (compound instanceof DifferenceExt) {
                if (index == 0) {
                    structuralStatement(subj, component, truthDed, memory);
                } else {
                    structuralStatement(subj, component, truthNDed, memory);
                }
            }
        } */
    }

    /// 模拟`StructuralRules.structuralStatement`
    /// * 📝结构化构建陈述，并发送到记忆区
    ///
    /// # 📄OpenNARS
    ///
    /// Common final operations of the above two methods
    ///
    /// @param subject   The subject of the new task
    /// @param predicate The predicate of the new task
    /// @param truth     The truth value of the new task
    /// @param memory    Reference to the memory
    fn __structural_statement(
        subject: &Term,
        predicate: &Term,
        truth: &C::Truth,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Term oldContent = task.getContent();
        if (oldContent instanceof Statement) {
            Term content = Statement.make((Statement) oldContent, subject, predicate, memory);
            if (content != null) {
                BudgetValue budget = BudgetFunctions.compoundForward(truth, content, memory);
                memory.singlePremiseTask(content, truth, budget);
            }
        } */
    }

    /* -------------------- set transform -------------------- */

    /// 模拟`StructuralRules.transformSetRelation`
    ///
    /// # 📄OpenNARS
    ///
    /// {<S --> {P}>} |- <S <-> {P}>
    ///
    /// @param compound  The set compound
    /// @param statement The premise
    /// @param side      The location of the indicated term in the premise
    /// @param memory    Reference to the memory
    fn transform_set_relation(
        compound: &Term,
        statement: &Term,
        side: SyllogismPosition,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (compound.size() > 1) {
            return;
        }
        if (statement instanceof Inheritance) {
            if (((compound instanceof SetExt) && (side == 0)) || ((compound instanceof SetInt) && (side == 1))) {
                return;
            }
        }
        Term sub = statement.getSubject();
        Term pre = statement.getPredicate();
        Term content;
        if (statement instanceof Inheritance) {
            content = Similarity.make(sub, pre, memory);
        } else {
            if (((compound instanceof SetExt) && (side == 0)) || ((compound instanceof SetInt) && (side == 1))) {
                content = Inheritance.make(pre, sub, memory);
            } else {
                content = Inheritance.make(sub, pre, memory);
            }
        }
        if (content == null) {
            return;
        }
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
        } else {
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    // ! 🚩【2024-05-21 22:03:09】目前有关NAL-4「单任务转换」的规则，均迁移至`transform_rules`中

    /* --------------- Disjunction and Conjunction transform --------------- */

    /// 模拟`StructuralRules.structuralCompound`
    /// * 📝合取、析取之「抽取」
    ///
    /// # 📄OpenNARS
    ///
    /// `{(&&, A, B), A@(&&, A, B)} |- A`, or answer `(&&, A, B)?` using `A`
    /// `{(||, A, B), A@(||, A, B)} |- A`, or answer `(||, A, B)?` using `A`
    ///
    /// @param compound     The premise
    /// @param component    The recognized component in the premise
    /// @param compoundTask Whether the compound comes from the task
    /// @param memory       Reference to the memory
    fn structural_compound(
        compound: &Term,
        component: &Term,
        compound_task: bool,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        if (!component.isConstant()) {
            return;
        }
        Term content = (compoundTask ? component : compound);
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
        } else {
            if ((sentence.isJudgment()) == (compoundTask == (compound instanceof Conjunction))) {
                truth = TruthFunctions.deduction(truth, RELIANCE);
            } else {
                TruthValue v1, v2;
                v1 = TruthFunctions.negation(truth);
                v2 = TruthFunctions.deduction(v1, RELIANCE);
                truth = TruthFunctions.negation(v2);
            }
            budget = BudgetFunctions.forward(truth, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    /* --------------- Negation related rules --------------- */

    /// 模拟`StructuralRules.transformNegation`
    /// * 📝否定的产生
    ///
    /// # 📄OpenNARS
    ///
    /// {A, A@(--, A)} |- (--, A)
    ///
    /// @param content The premise
    /// @param memory  Reference to the memory
    fn transform_negation(content: &Term, memory: &mut C::Memory) {
        /* 📄OpenNARS源码：
        Task task = memory.currentTask;
        Sentence sentence = task.getSentence();
        TruthValue truth = sentence.getTruth();
        if (sentence.isJudgment()) {
            truth = TruthFunctions.negation(truth);
        }
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
        } else {
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    /// 模拟`StructuralRules.contraposition`
    /// * 📝双重否定规则
    ///
    /// # 📄OpenNARS
    ///
    /// {<A ==> B>, A@(--, A)} |- <(--, B) ==> (--, A)>
    ///
    /// @param statement The premise
    /// @param memory    Reference to the memory
    fn contraposition(statement: &Term, memory: &mut C::Memory) {
        /* 📄OpenNARS源码：
        Term subj = statement.getSubject();
        Term pred = statement.getPredicate();
        Term content = Statement.make(statement, Negation.make(pred, memory), Negation.make(subj, memory), memory);
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            if (content instanceof Implication) {
                budget = BudgetFunctions.compoundBackwardWeak(content, memory);
            } else {
                budget = BudgetFunctions.compoundBackward(content, memory);
            }
            memory.singlePremiseTask(content, Symbols.QUESTION_MARK, truth, budget);
        } else {
            if (content instanceof Implication) {
                truth = TruthFunctions.contraposition(truth);
            }
            budget = BudgetFunctions.compoundForward(truth, content, memory);
            memory.singlePremiseTask(content, Symbols.JUDGMENT_MARK, truth, budget);
        } */
    }
}

/// 自动实现，以便添加方法
impl<C: TypeContext, T: DerivationContext<C>> StructuralRules<C> for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
