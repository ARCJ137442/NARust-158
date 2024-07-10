//! 🎯复刻OpenNARS `nars.inference.RuleTables`
//! * 📌「概念推理」的入口函数
//! * 📝规则分派的起始点
//!
//! ## Logs
//!
//! * ♻️【2024-07-10 21:44:07】开始根据改版OpenNARS重写

use crate::{
    control::{ReasonContext, ReasonContextConcept, ReasonContextWithLinks},
    entity::{Judgement, Sentence, TLink, TLinkType},
    language::{CompoundTerm, Statement, Term},
    util::RefCount,
};

/// 存储规则表之外的结构与方法
mod utils {
    /// 🆕三段论位置
    /// * 🎯用于表征[`RuleTables::index_to_figure`]推导出的「三段论子类型」
    /// * 📝OpenNARS中是在「三段论推理」的「陈述🆚陈述」中表示「位置关系」
    ///   * 📄`<A --> B>`与`<B --> C>`中，`B`就分别在`1`、`0`两个索引位置
    ///     * 📌因此有`SP`或`Subject-Predicate`
    ///     * 📌同时也有了其它三种「三段论图式」
    /// * 🚩两种情况：
    ///   * 主项
    ///   * 谓项
    #[doc(alias = "SyllogismLocation")]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum SyllogismPosition {
        /// 主项（第一项）
        Subject = 0,
        /// 谓项（第二项）
        Predicate = 1,
    }

    impl SyllogismPosition {
        /// 🆕调转到相反位置
        pub fn opposite(self) -> Self {
            match self {
                Subject => Predicate,
                Predicate => Subject,
            }
        }

        /// 🆕从「数组索引」中来
        /// * 🎯[`RuleTables::__index_to_figure`]
        /// * 🚩核心：0→主项，1→谓项，整体`<主项 --> 谓项>`
        pub fn from_index(index: usize) -> Self {
            match index {
                0 => Subject,
                1 => Predicate,
                _ => panic!("无效索引"),
            }
        }

        /// 🆕构造「三段论图式」
        /// * 🎯[`RuleTables::__index_to_figure`]
        /// * 🚩直接构造二元组
        pub fn build_figure(first: Self, second: Self) -> SyllogismFigure {
            (first, second)
        }
    }
    use SyllogismPosition::*;

    /// 三段论图式
    /// * 🎯模拟「三段论推理」中「公共项在两陈述的位置」的四种情况
    /// * 🚩使用二元组实现，允许更细化的组合
    ///   * ✨基本等同于整数（低开销）类型
    /// * 📝四种主要情况：
    ///   * 主项-主项
    ///   * 主项-谓项
    ///   * 谓项-主项
    ///   * 谓项-谓项
    ///
    /// # 📄OpenNARS
    ///
    /// location of the shared term
    pub type SyllogismFigure = (SyllogismPosition, SyllogismPosition);

    /// 存储「三段论图式」常量
    /// * 🎯可完全引用，可简短使用
    ///   * ⚡长度与OpenNARS的`11`、`12`相近
    /// * 🚩仅四种
    pub mod syllogistic_figures {
        use super::*;

        /// [三段论图式](SyllogismFigure)/常用/主项-主项
        #[doc(alias = "SUBJECT_SUBJECT")]
        pub const SS: SyllogismFigure = (Subject, Subject);

        /// [三段论图式](SyllogismFigure)/常用/主项-谓项
        #[doc(alias = "SUBJECT_PREDICATE")]
        pub const SP: SyllogismFigure = (Subject, Predicate);

        /// [三段论图式](SyllogismFigure)/常用/谓项-主项
        #[doc(alias = "PREDICATE_SUBJECT")]
        pub const PS: SyllogismFigure = (Predicate, Subject);

        /// [三段论图式](SyllogismFigure)/常用/谓项-谓项
        #[doc(alias = "PREDICATE_PREDICATE")]
        pub const PP: SyllogismFigure = (Predicate, Predicate);
    }

    /// 三段论推理中的「某侧」
    /// * 📌包含「主项/谓项/整个词项」
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum SyllogismSide {
        /// 主项（第一项）
        Subject = 0,
        /// 谓项（第二项）
        Predicate = 1,
        /// 整个词项（整体）
        Whole = -1,
    }
}
pub use utils::*;

/// 在断言的情况下，从[`Term`]中提取[`CompoundTerm`]
/// * 🎯对标OpenNARS`(CompoundTerm) term`的转换
fn cast_compound(term: Term) -> CompoundTerm {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_compound(),
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 在断言的情况下，从[`Term`]中提取[`Statement`]
/// * 🎯对标OpenNARS`(Statement) term`的转换
fn cast_statement(term: Term) -> Statement {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_statement(),
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 模拟`RuleTables.reason`
///
/// # 📄OpenNARS
///
/// Entry point of the inference engine
///
/// @param tLink  The selected TaskLink, which will provide a task
/// @param bLink  The selected TermLink, which may provide a belief
/// @param memory Reference to the memory
pub fn reason(context: &mut ReasonContextConcept) {
    // * 🚩提取参数
    let t_link = context.current_task_link();
    let b_link = context.current_belief_link();
    let task_rc = context.current_task();
    let task = task_rc.get_();
    let task_sentence = task.sentence_clone(); // 复制语句以避免借用问题
    let belief = context.current_belief().cloned(); // 复制语句以避免借用问题
    let mut concept_term = context.current_concept().term().clone(); // cloning for substitution
    let mut task_term = task.content().clone(); // cloning for substitution
    let mut belief_term = context.current_belief_link().target().clone(); // cloning for substitution
    drop(task);
    drop(task_rc);

    // * 📝词项链所指的词项，不一定指向一个确切的「信念」（并非「语句链」）
    let t_index = t_link.get_index(0).cloned(); // 复制以避免借用问题
    let b_index = b_link.get_index(0).cloned(); // 复制以避免借用问题
    let t_link_type = t_link.link_type();
    let b_link_type = b_link.link_type();

    // * 🚩直接一个match分派好
    use TLinkType::*;
    match [t_link_type, b_link_type] {
        // * 🚩↓已经在转换推理中处理过
        [Transform, _] | [_, Transform] => { /* 不可能 */ }

        // * conceptTerm = taskTerm * //

        // * 📝【2024-07-10 22:28:32】OpenNARS不存在
        [SELF, SELF] => {}

        // * 📄T="(&&,<#1 --> object>,<#1 --> (/,made_of,_,plastic)>)"
        // * + B="object"
        // * @ C="(&&,<#1 --> object>,<#1 --> (/,made_of,_,plastic)>)"
        [SELF, Component] => {
            compound_and_self(cast_compound(task_term), belief_term, true, context)
        }

        // * 📄T="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        // * + B="[aggressive]"
        // * @ C="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        [SELF, Compound] => {
            compound_and_self(cast_compound(belief_term), task_term, false, context)
        }

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="{tim}"
        // * @ C="<{tim} --> (/,livingIn,_,{graz})>"
        [SELF, ComponentStatement] => {
            if let Some(belief) = belief {
                // SyllogisticRules.detachment(task, belief, bIndex, context);
            }
        }

        // *📄T="<{tim} --> (/,own,_,sunglasses)>"
        // * + B="<<{tim} --> (/,own,_,sunglasses)> ==> <{tim} --> murder>>"
        // * @ C=T
        [SELF, CompoundStatement] => {
            if let Some(belief) = belief {
                // SyllogisticRules.detachment(belief, task, bIndex, context);
            }
        }

        // *📄T="<(&&,<$1-->[aggressive]>,<$1-->(/,livingIn,_,{graz})>)==><$1-->murder>>"
        // * + B="[aggressive]"
        // * @ C=T
        [SELF, ComponentCondition] => {
            if let Some(belief) = belief {
                // * 📝「复合条件」一定有两层，就处在作为「前件」的「条件」中
                /* SyllogisticRules.conditionalDedInd(
                (Implication) taskTerm, bLink.getIndex(1),
                beliefTerm, tIndex,
                context); */
            }
        }

        // * 📄T="<(*,{tim},{graz}) --> livingIn>"
        // * + B="<(&&,<{tim} --> [aggressive]>,<(*,{tim},{graz}) --> livingIn>) ==> <{tim} --> murder>>"
        // * @ C=T
        [SELF, CompoundCondition] => {
            // ! ❌【2024-06-18 21:34:08】「任务链是「复合条件」的，当前任务一定是复合词项」不一定成立
            // * 📄edge case：
            // * * task="flyer"
            // * * belief="<(&&,<$1 --> flyer>,<(*,$1,worms) --> food>) ==> <$1 --> bird>>"
            if let Some(belief) = belief {
                // * 📝「复合条件」一定有两层，就处在作为「前件」的「条件」中
                /* SyllogisticRules.conditionalDedInd(
                (Implication) beliefTerm, bLink.getIndex(1),
                taskTerm, tIndex,
                context); */
            }
        }

        // * 📝【2024-07-10 22:32:16】OpenNARS均不存在
        [Component, _] => {}

        // * conceptTerm ∈ taskTerm * //
        [Compound, SELF] => {}

        [Compound, Component] => {}

        // * 🚩conceptTerm ∈ taskTerm, conceptTerm ∈ beliefTerm
        // * 📄T="(&&,<cup --> #1>,<toothbrush --> #1>)"
        // * + B="<cup --> [bendable]>"
        // * @ C="cup"
        [Compound, Compound] => {
            /* compoundAndCompound(
            (CompoundTerm) taskTerm,
            (CompoundTerm) beliefTerm,
            context); */
        }

        [Compound, ComponentStatement] => {}

        // * 🚩conceptTerm ∈ taskTerm, conceptTerm ∈ beliefTerm (statement)
        // * 📄T="(&&,<{tim} --> #1>,<{tom} --> #1>)"
        // * + B="<{tom} --> murder>"
        // * @ C="{tom}"
        [Compound, CompoundStatement] => {
            /* compoundAndStatement(
            (CompoundTerm) taskTerm, tIndex,
            (Statement) beliefTerm, bIndex,
            beliefTerm, context); */
        }

        [Compound, ComponentCondition] => {}

        // *📄T="(||,<{tom}-->[aggressive]>,<{tom}-->(/,livingIn,_,{graz})>)"
        // * + B="<(&&,<$1-->[aggressive]>,<$1-->(/,livingIn,_,{graz})>)==><$1-->murder>>"
        // * @ C="(/,livingIn,_,{graz})"
        [Compound, CompoundCondition] => {
            /* reason_compoundAndCompoundCondition(
            context,
            task, (CompoundTerm) taskTerm,
            belief, (Implication) beliefTerm,
            bIndex); */
        }

        // * 📝【2024-07-10 22:37:22】OpenNARS均不存在
        [ComponentStatement, _] => {}

        // * conceptTerm ∈ taskTerm (statement) * //
        [CompoundStatement, SELF] => {}

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="tim"
        // * @ C="{tim}"
        [CompoundStatement, Component] => {
            /* componentAndStatement(
            (CompoundTerm) conceptTerm, bIndex,
            (Statement) taskTerm,
            tIndex,
            context); */
        }

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="{tim}"
        // * @ C="tim"
        [CompoundStatement, Compound] => {
            /* compoundAndStatement(
            (CompoundTerm) beliefTerm, bIndex,
            (Statement) taskTerm, tIndex,
            beliefTerm, context); */
        }

        [CompoundStatement, ComponentStatement] => {}

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="<<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>"
        // * @ C="(/,livingIn,_,{graz})"
        [CompoundStatement, CompoundStatement] => {
            if let Some(belief) = belief {
                syllogisms(
                    /* t_link, b_link, */
                    cast_statement(task_term),
                    cast_statement(belief_term),
                    belief,
                    context,
                )
            }
        }

        [CompoundStatement, ComponentCondition] => {}

        // * 📄T="<<$1 --> [aggressive]> ==> <$1 --> (/,livingIn,_,{graz})>>"
        // *+B="<(&&,<$1-->[aggressive]>,<$1-->(/,livingIn,_,{graz})>)==><$1-->murder>>"
        // * @ C="(/,livingIn,_,{graz})"
        [CompoundStatement, CompoundCondition] => {
            if let Some(belief) = belief {
                /* conditionalDedIndWithVar(
                // * 🚩获取「信念链」内部指向的复合词项
                // * 📝「复合条件」一定有两层，就处在作为「前件」的「条件」中
                (Implication) beliefTerm, bLink.getIndex(1),
                (Statement) taskTerm,
                tIndex, context); */
            }
        }

        // * 📝【2024-07-10 23:08:10】OpenNARS均不出现
        [ComponentCondition, _] => {}

        // * conceptTerm ∈ taskTerm (condition in statement) * //
        [CompoundCondition, SELF] => {}

        [CompoundCondition, Component] => {}

        // * 📄T="<(&&,<{graz} --> (/,livingIn,$1,_)>,(||,<$1 --> [aggressive]>,<sunglasses --> (/,own,$1,_)>)) ==> <$1 --> murder>>"
        // * + B="(/,livingIn,_,{graz})"
        // * @ C="{graz}"
        [CompoundCondition, Compound] => {
            if let Some(belief) = belief { /* detachmentWithVar(task, belief, tIndex, context); */ }
        }

        [CompoundCondition, ComponentStatement] => {}

        // *📄T="<(&&,<$1-->[aggressive]>,<sunglasses-->(/,own,$1,_)>)==><$1-->murder>>"
        // * + B="<sunglasses --> glasses>"
        // * @ C="sunglasses"
        [CompoundCondition, CompoundStatement] => {
            if let Some(belief) = belief {
                /* compoundConditionAndCompoundStatement(
                context,
                task, (Implication) taskTerm, tIndex,
                belief, (Statement) beliefTerm, bIndex); */
            }
        }

        [CompoundCondition, ComponentCondition] => {}

        [CompoundCondition, CompoundCondition] => {}
    }
}

fn syllogisms(
    task_term: Statement,
    belief_term: Statement,
    belief: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // * 🚩提取参数
    let t_link = context.current_task_link();
    let b_link = context.current_belief_link();
    context.report_comment(format!(
        "TODO @ syllogisms: \ntask_term={task_term}\nbelief_term={belief_term}\nbelief={}",
        belief.to_display_long()
    ))
}

fn compound_and_self(
    compound: CompoundTerm,
    component: Term,
    is_compound_from_task: bool,
    context: &mut ReasonContextConcept,
) {
    context.report_comment(format!("TODO @ compound_and_self: \ncompound={compound}\ncomponent={component}\nis_compound_from_task={is_compound_from_task}"))
}
