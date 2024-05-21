//! 🆕提取OpenNARS中「概念+任务链」的「单链推理机制」
//! * 📄自`nars.inference.RuleTables`与`nars.inference.StructuralRules`中各拿取部分函数功能
//!
//! * ✅【2024-05-11 10:08:34】初步复现方法API
//!
//! TODO: 完成具体实现

use crate::{control::*, language::Term, types::TypeContext};

/// 规则表@「转换推理」
/// * 🎯用于「概念+单任务链」在「仅有单链」的情况下推理
/// * 🎯直接对标[`DerivationContextTransform`]
/// * 📌入口函数：[`TransformRules::transform_task`]
pub trait TransformRules<C: TypeContext>: DerivationContextTransform<C> {
    /* ----- inference with one TaskLink only ----- */

    /// 模拟`RuleTables.transformTask`
    /// * 🚩【2024-05-08 16:36:34】仅保留「记忆区」单个参数
    ///   * 📌情况：该函数只在[`Memory::__fire_concept`]调用，且其中的`task_link`也固定为「当前任务链」
    ///   * 📌原因：同时传入「自身可变引用」与「自身不可变引用」⇒借用错误
    /// * 🚩【2024-05-18 01:04:04】该方法仅用于「直接推理」
    ///
    /// # 📄OpenNARS
    ///
    /// The TaskLink is of type TRANSFORM, and the conclusion is an equivalent
    /// transformation
    ///
    /// @param tLink  The task link
    /// @param memory Reference to the memory
    fn transform_task(/* task_link: &C::TaskLink,  */ &mut self) {
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
        let task_link = self.current_task_link();
        todo!("// TODO: 有待实现")
    }

    /* -------------------- products and images transform -------------------- */
    // TODO: 【2024-05-11 14:55:48】确认并理清其中`relation_index`与`placeholder_index`的关系（是否等价？是否可以直接拿来用？）

    /// 模拟`StructuralRules.transformProductImage`
    ///
    /// # 📄OpenNARS
    ///
    /// Equivalent transformation between products and images
    /// {<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>
    /// {<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>
    /// {<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>
    ///
    /// @param inh        An Inheritance statement
    /// @param oldContent The whole content
    /// @param indices    The indices of the TaskLink
    /// @param task       The task
    /// @param memory     Reference to the memory
    fn transform_product_image(
        inh: &Term,
        old_content: &Term,
        indices: &[usize],
        task: &C::Task,
        memory: &mut C::Memory,
    ) {
        /* 📄OpenNARS源码：
        Term subject = inh.getSubject();
        Term predicate = inh.getPredicate();
        if (inh.equals(oldContent)) {
            if (subject instanceof CompoundTerm) {
                transformSubjectPI((CompoundTerm) subject, predicate, memory);
            }
            if (predicate instanceof CompoundTerm) {
                transformPredicatePI(subject, (CompoundTerm) predicate, memory);
            }
            return;
        }
        short index = indices[indices.length - 1];
        short side = indices[indices.length - 2];
        CompoundTerm comp = (CompoundTerm) inh.componentAt(side);
        if (comp instanceof Product) {
            if (side == 0) {
                subject = comp.componentAt(index);
                predicate = ImageExt.make((Product) comp, inh.getPredicate(), index, memory);
            } else {
                subject = ImageInt.make((Product) comp, inh.getSubject(), index, memory);
                predicate = comp.componentAt(index);
            }
        } else if ((comp instanceof ImageExt) && (side == 1)) {
            if (index == ((ImageExt) comp).getRelationIndex()) {
                subject = Product.make(comp, inh.getSubject(), index, memory);
                predicate = comp.componentAt(index);
            } else {
                subject = comp.componentAt(index);
                predicate = ImageExt.make((ImageExt) comp, inh.getSubject(), index, memory);
            }
        } else if ((comp instanceof ImageInt) && (side == 0)) {
            if (index == ((ImageInt) comp).getRelationIndex()) {
                subject = comp.componentAt(index);
                predicate = Product.make(comp, inh.getPredicate(), index, memory);
            } else {
                subject = ImageInt.make((ImageInt) comp, inh.getPredicate(), index, memory);
                predicate = comp.componentAt(index);
            }
        } else {
            return;
        }
        Inheritance newInh = Inheritance.make(subject, predicate, memory);
        Term content = null;
        if (indices.length == 2) {
            content = newInh;
        } else if ((oldContent instanceof Statement) && (indices[0] == 1)) {
            content = Statement.make((Statement) oldContent, oldContent.componentAt(0), newInh, memory);
        } else {
            ArrayList<Term> componentList;
            Term condition = oldContent.componentAt(0);
            if (((oldContent instanceof Implication) || (oldContent instanceof Equivalence))
                    && (condition instanceof Conjunction)) {
                componentList = ((CompoundTerm) condition).cloneComponents();
                componentList.set(indices[1], newInh);
                Term newCond = CompoundTerm.make((CompoundTerm) condition, componentList, memory);
                content = Statement.make((Statement) oldContent, newCond, ((Statement) oldContent).getPredicate(),
                        memory);
            } else {
                componentList = oldContent.cloneComponents();
                componentList.set(indices[0], newInh);
                if (oldContent instanceof Conjunction) {
                    content = CompoundTerm.make(oldContent, componentList, memory);
                } else if ((oldContent instanceof Implication) || (oldContent instanceof Equivalence)) {
                    content = Statement.make((Statement) oldContent, componentList.get(0), componentList.get(1),
                            memory);
                }
            }
        }
        if (content == null) {
            return;
        }
        Sentence sentence = memory.currentTask.getSentence();
        TruthValue truth = sentence.getTruth();
        BudgetValue budget;
        if (sentence.isQuestion()) {
            budget = BudgetFunctions.compoundBackward(content, memory);
        } else {
            budget = BudgetFunctions.compoundForward(truth, content, memory);
        }
        memory.singlePremiseTask(content, truth, budget); */
    }

    /// 模拟`StructuralRules.transformSubjectPI`
    ///
    /// # 📄OpenNARS
    ///
    /// Equivalent transformation between products and images when the subject is a compound
    /// {<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>
    /// {<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>
    /// {<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>
    ///
    /// @param subject   The subject term
    /// @param predicate The predicate term
    /// @param memory    Reference to the memory
    fn __transform_subject_pi(subject: &Term, predicate: &Term, memory: &mut C::Memory) {
        /* 📄OpenNARS源码：
        TruthValue truth = memory.currentTask.getSentence().getTruth();
        BudgetValue budget;
        Inheritance inheritance;
        Term newSubj, newPred;
        if (subject instanceof Product) {
            Product product = (Product) subject;
            for (short i = 0; i < product.size(); i++) {
                newSubj = product.componentAt(i);
                newPred = ImageExt.make(product, predicate, i, memory);
                inheritance = Inheritance.make(newSubj, newPred, memory);
                if (inheritance != null) {
                    if (truth == null) {
                        budget = BudgetFunctions.compoundBackward(inheritance, memory);
                    } else {
                        budget = BudgetFunctions.compoundForward(truth, inheritance, memory);
                    }
                    memory.singlePremiseTask(inheritance, truth, budget);
                }
            }
        } else if (subject instanceof ImageInt) {
            ImageInt image = (ImageInt) subject;
            int relationIndex = image.getRelationIndex();
            for (short i = 0; i < image.size(); i++) {
                if (i == relationIndex) {
                    newSubj = image.componentAt(relationIndex);
                    newPred = Product.make(image, predicate, relationIndex, memory);
                } else {
                    newSubj = ImageInt.make((ImageInt) image, predicate, i, memory);
                    newPred = image.componentAt(i);
                }
                inheritance = Inheritance.make(newSubj, newPred, memory);
                if (inheritance != null) {
                    if (truth == null) {
                        budget = BudgetFunctions.compoundBackward(inheritance, memory);
                    } else {
                        budget = BudgetFunctions.compoundForward(truth, inheritance, memory);
                    }
                    memory.singlePremiseTask(inheritance, truth, budget);
                }
            }
        } */
    }

    /// 模拟`StructuralRules.transformPredicatePI`
    ///
    /// # 📄OpenNARS
    ///
    /// Equivalent transformation between products and images when the predicate is a compound
    /// {<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>
    /// {<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>
    /// {<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>
    ///
    /// @param subject   The subject term
    /// @param predicate The predicate term
    /// @param memory    Reference to the memory
    fn __transform_predicate_pi(subject: &Term, predicate: &Term, memory: &mut C::Memory) {
        /* 📄OpenNARS源码：
        TruthValue truth = memory.currentTask.getSentence().getTruth();
        BudgetValue budget;
        Inheritance inheritance;
        Term newSubj, newPred;
        if (predicate instanceof Product) {
            Product product = (Product) predicate;
            for (short i = 0; i < product.size(); i++) {
                newSubj = ImageInt.make(product, subject, i, memory);
                newPred = product.componentAt(i);
                inheritance = Inheritance.make(newSubj, newPred, memory);
                if (inheritance != null) {
                    if (truth == null) {
                        budget = BudgetFunctions.compoundBackward(inheritance, memory);
                    } else {
                        budget = BudgetFunctions.compoundForward(truth, inheritance, memory);
                    }
                    memory.singlePremiseTask(inheritance, truth, budget);
                }
            }
        } else if (predicate instanceof ImageExt) {
            ImageExt image = (ImageExt) predicate;
            int relationIndex = image.getRelationIndex();
            for (short i = 0; i < image.size(); i++) {
                if (i == relationIndex) {
                    newSubj = Product.make(image, subject, relationIndex, memory);
                    newPred = image.componentAt(relationIndex);
                } else {
                    newSubj = image.componentAt(i);
                    newPred = ImageExt.make((ImageExt) image, subject, i, memory);
                }
                inheritance = Inheritance.make(newSubj, newPred, memory);
                if (inheritance != null) { // jmv <<<<<
                    if (truth == null) {
                        budget = BudgetFunctions.compoundBackward(inheritance, memory);
                    } else {
                        budget = BudgetFunctions.compoundForward(truth, inheritance, memory);
                    }
                    memory.singlePremiseTask(inheritance, truth, budget);
                }
            }
        } */
    }
}
