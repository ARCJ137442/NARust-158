//! 有关「概念推理」中与「链接」相关的部分
//! * 🎯承载NARS控制机制中有关「词项链/任务链」的机制
//!   * 📌链接到任务(内部, 生成词项链/任务链) from 直接处理(@主模块)
//!   * 📌插入任务链(非外部) from 链接到任务
//!   * 📌构建词项链(非外部) from 链接到任务
//!   * 📌预备词项链模板(仅复合词项) from `构造函数`
//!   * 📌插入词项链(非外部) from 构建词项链
//! * 📄仿自OpenNARS 3.0.4
//!
//! * ♻️【2024-05-16 18:07:08】初步独立成模块功能

use crate::{control::*, entity::*, inference::*, language::Term, storage::*, types::TypeContext};

///
/// * 🚩因为`<Self as LocalRules>::solution_quality`要求[`Sized`]
pub trait ConceptProcessLink<C: TypeContext>: DerivationContext<C> {
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
    fn __link_to_task(&mut self, task: &mut C::Task) {
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
    fn insert_task_link(&mut self, task_link: C::TaskLink) {
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
    fn build_term_links(&mut self, task_budget: &C::Budget) {
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
    fn prepare_component_link_templates(self_term: &Term) -> Vec<C::TermLink> {
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
        component_links: &mut Vec<C::TermLink>,
        type_: &TermLinkType,
        term: &Term,
    ) -> Vec<C::TermLink> {
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
    fn insert_term_link(&mut self, term_link: C::TermLink, concept: &mut C::Concept) {
        /* 📄OpenNARS源码：
        termLinks.putIn(termLink); */
        concept.__term_links_mut().put_in(term_link);
    }
}

/// 自动实现，以便添加方法
impl<C: TypeContext, T: DerivationContext<C>> ConceptProcessLink<C> for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
