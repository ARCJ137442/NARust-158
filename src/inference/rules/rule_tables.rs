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
    inference::{syllogisms, PremiseSource, SyllogismPosition, SyllogismSide},
    language::{CompoundTerm, Statement, Term},
    util::RefCount,
};

/// 在断言的情况下，从[`Term`]中提取[`CompoundTerm`]
/// * 🎯对标OpenNARS`(CompoundTerm) term`的转换
pub(super) fn cast_compound(term: Term) -> CompoundTerm {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_compound(),
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 在断言的情况下，从[`Term`]中提取[`Statement`]
/// * 🎯对标OpenNARS`(Statement) term`的转换
pub(super) fn cast_statement(term: Term) -> Statement {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_statement(),
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 模拟`RuleTables.reason`
/// * 📌规则表入口
/// * 📝「概念推理」的起点
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
    let concept_term = context.current_concept().term().clone(); // cloning for substitution
    let task_term = task.content().clone(); // cloning for substitution
    let belief_term = context.current_belief_link().target().clone(); // cloning for substitution
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
        [SELF, Component] => compound_and_self(
            cast_compound(task_term),
            belief_term,
            super::syllogistic_rules::PremiseSource::Task,
            context,
        ),

        // * 📄T="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        // * + B="[aggressive]"
        // * @ C="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        [SELF, Compound] => compound_and_self(
            cast_compound(belief_term),
            task_term,
            super::syllogistic_rules::PremiseSource::Belief,
            context,
        ),

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="{tim}"
        // * @ C="<{tim} --> (/,livingIn,_,{graz})>"
        [SELF, ComponentStatement] => {
            if let Some(belief) = belief {
                super::syllogistic_rules::detachment(
                    &task_sentence,
                    &belief,
                    super::syllogistic_rules::PremiseSource::Task,
                    SyllogismPosition::from_index(b_index.unwrap()),
                    context,
                )
            }
        }

        // *📄T="<{tim} --> (/,own,_,sunglasses)>"
        // * + B="<<{tim} --> (/,own,_,sunglasses)> ==> <{tim} --> murder>>"
        // * @ C=T
        [SELF, CompoundStatement] => {
            if let Some(belief) = belief {
                super::syllogistic_rules::detachment(
                    &task_sentence,
                    &belief,
                    super::syllogistic_rules::PremiseSource::Belief,
                    SyllogismPosition::from_index(b_index.unwrap()),
                    context,
                )
            }
        }

        // *📄T="<(&&,<$1-->[aggressive]>,<$1-->(/,livingIn,_,{graz})>)==><$1-->murder>>"
        // * + B="[aggressive]"
        // * @ C=T
        [SELF, ComponentCondition] => {
            if let Some(belief) = belief {
                // * 📝「复合条件」一定有两层，就处在作为「前件」的「条件」中
                super::syllogistic_rules::conditional_ded_ind(
                    cast_statement(task_term),
                    *b_link.get_index(1).unwrap(),
                    belief_term,
                    belief,
                    crate::inference::PremiseSource::Task,
                    SyllogismSide::from_index(t_index),
                    context,
                )
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
                super::syllogistic_rules::conditional_ded_ind(
                    cast_statement(belief_term),
                    *b_link.get_index(1).unwrap(),
                    task_term,
                    belief,
                    crate::inference::PremiseSource::Belief,
                    SyllogismSide::from_index(t_index),
                    context,
                )
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
        [Compound, Compound] => compound_and_compound(
            cast_compound(task_term),
            cast_compound(belief_term),
            context,
        ),

        [Compound, ComponentStatement] => {}

        // * 🚩conceptTerm ∈ taskTerm, conceptTerm ∈ beliefTerm (statement)
        // * 📄T="(&&,<{tim} --> #1>,<{tom} --> #1>)"
        // * + B="<{tom} --> murder>"
        // * @ C="{tom}"
        [Compound, CompoundStatement] => compound_and_statement(
            task_term == belief_term,
            super::syllogistic_rules::PremiseSource::Task,
            cast_compound(task_term),
            t_index.unwrap(),
            cast_statement(belief_term),
            SyllogismPosition::from_index(b_index.unwrap()),
            context,
        ),

        [Compound, ComponentCondition] => {}

        // *📄T="(||,<{tom}-->[aggressive]>,<{tom}-->(/,livingIn,_,{graz})>)"
        // * + B="<(&&,<$1-->[aggressive]>,<$1-->(/,livingIn,_,{graz})>)==><$1-->murder>>"
        // * @ C="(/,livingIn,_,{graz})"
        [Compound, CompoundCondition] => {
            if let Some(belief) = belief {
                compound_and_compound_condition(
                    task_sentence,
                    belief,
                    cast_compound(task_term),
                    cast_statement(belief_term),
                    SyllogismPosition::from_index(b_index.unwrap()),
                    context,
                )
            }
        }

        // * 📝【2024-07-10 22:37:22】OpenNARS均不存在
        [ComponentStatement, _] => {}

        // * conceptTerm ∈ taskTerm (statement) * //
        [CompoundStatement, SELF] => {}

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="tim"
        // * @ C="{tim}"
        [CompoundStatement, Component] => component_and_statement(
            task_term == belief_term,
            true,
            cast_compound(concept_term),
            b_index.unwrap(),
            cast_statement(task_term),
            SyllogismPosition::from_index(t_index.unwrap()),
            context,
        ),

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="{tim}"
        // * @ C="tim"
        [CompoundStatement, Compound] => compound_and_statement(
            task_term == belief_term,
            super::syllogistic_rules::PremiseSource::Belief,
            cast_compound(belief_term.clone()),
            b_index.unwrap(),
            cast_statement(task_term),
            SyllogismPosition::from_index(t_index.unwrap()),
            context,
        ),

        [CompoundStatement, ComponentStatement] => {}

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="<<$1 --> (/,livingIn,_,{graz})> ==> <$1 --> murder>>"
        // * @ C="(/,livingIn,_,{graz})"
        [CompoundStatement, CompoundStatement] => {
            if let Some(belief) = belief {
                syllogisms(
                    cast_statement(task_term),
                    cast_statement(belief_term),
                    t_index.expect("T链接索引越界@三段论推理"),
                    b_index.expect("T链接索引越界@三段论推理"),
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
                conditional_ded_ind_with_var(
                    PremiseSource::Belief,
                    // * 🚩获取「信念链」内部指向的复合词项
                    // * 📝「复合条件」一定有两层，就处在作为「前件」的「条件」中
                    cast_statement(belief_term),
                    b_link.get_index(1).cloned().unwrap(),
                    cast_statement(task_term),
                    SyllogismPosition::from_index(t_index.unwrap()),
                    belief,
                    context,
                )
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
            if let Some(belief) = belief {
                super::syllogistic_rules::detachment_with_var(
                    task_sentence,
                    belief,
                    super::syllogistic_rules::PremiseSource::Task,
                    SyllogismPosition::from_index(t_index.unwrap()),
                    context,
                )
            }
        }

        [CompoundCondition, ComponentStatement] => {}

        // *📄T="<(&&,<$1-->[aggressive]>,<sunglasses-->(/,own,$1,_)>)==><$1-->murder>>"
        // * + B="<sunglasses --> glasses>"
        // * @ C="sunglasses"
        [CompoundCondition, CompoundStatement] => {
            if let Some(belief) = belief {
                compound_condition_and_compound_statement(
                    task_sentence,
                    cast_statement(task_term),
                    SyllogismPosition::from_index(t_index.unwrap()),
                    belief,
                    cast_statement(belief_term),
                    SyllogismPosition::from_index(b_index.unwrap()),
                    context,
                )
            }
        }

        [CompoundCondition, ComponentCondition] => {}

        [CompoundCondition, CompoundCondition] => {}
    }
}

/// 分派：复合词项与其元素
///
/// # 📄OpenNARS
///
/// Inference between a compound term and a component of it
fn compound_and_self(
    compound: CompoundTerm,
    component: Term,
    where_compound_from: super::syllogistic_rules::PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // TODO
    context.report_comment(format!("TODO @ compound_and_self: \ncompound={compound}\ncomponent={component}\nwhere_compound_from={where_compound_from:?}"))
}

/// 分派：复合词项与复合词项
///
/// # 📄OpenNARS
///
/// Inference between two compound terms
fn compound_and_compound(
    task_term: CompoundTerm,
    belief_term: CompoundTerm,
    context: &mut ReasonContextConcept,
) {
    // * 🚩非同类⇒返回
    if !task_term.is_same_type(&belief_term) {
        return;
    }
    use super::syllogistic_rules::PremiseSource::*;
    use std::cmp::Ordering::*;
    match task_term
        .get_ref()
        .size()
        .cmp(&belief_term.get_ref().size())
    {
        // * 🚩任务词项 > 信念词项 ⇒ 以「任务词项」为整体
        Greater => compound_and_self(task_term, belief_term.into(), Task, context),
        // * 🚩任务词项 < 信念词项 ⇒ 以「信念词项」为整体
        Less => compound_and_self(belief_term, task_term.into(), Belief, context),
        // * 🚩其它情况 ⇒ 返回
        _ => {}
    }
}

/// 分派：复合词项与陈述
///
/// # 📄OpenNARS
///
/// Inference between a compound term and a statement
fn compound_and_statement(
    statement_equals_belief: bool,
    compound_from: super::syllogistic_rules::PremiseSource,
    compound: CompoundTerm,
    index: usize,
    statement: Statement,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 分派：复合词项与陈述
///
/// # 📄OpenNARS
///
/// Inference between a compound term and a statement
fn component_and_statement(
    statement_equals_belief: bool,
    compound_from_concept: bool,
    compound: CompoundTerm,
    index: usize,
    statement: Statement,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 分派：复合词项×复合条件
fn compound_and_compound_condition(
    task_sentence: impl Sentence,
    belief: impl Judgement,
    task_term: CompoundTerm,
    belief_term: Statement,
    b_index: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 分派：条件演绎/归纳 & 变量
fn conditional_ded_ind_with_var(
    conditional_from: PremiseSource,
    conditional: Statement,
    index: usize,
    statement: Statement,
    side: SyllogismPosition,
    belief: impl Judgement,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

fn compound_condition_and_compound_statement(
    task_sentence: impl Sentence,
    task_term: Statement,
    t_side: SyllogismPosition,
    belief: crate::entity::JudgementV1,
    belief_term: Statement,
    b_side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 一些通用函数
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::inference::{process_direct, transform_task, InferenceEngine};

    /// 概念推理专用测试引擎
    /// * 🚩【2024-07-14 23:51:32】禁掉了转换推理
    pub const ENGINE_REASON: InferenceEngine = InferenceEngine::new(
        process_direct,
        transform_task,
        InferenceEngine::VOID.matching_f(),
        reason,
    );
}
