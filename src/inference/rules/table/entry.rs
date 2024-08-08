//! 规则表入口
//! * 📝规则分派的起始点
//! * 🎯负责所有规则的分派入口

use super::syllogistic::*;
use crate::{
    control::*,
    entity::*,
    inference::rules::{utils::*, *},
    io::symbols::{IMPLICATION_RELATION, INHERITANCE_RELATION, SIMILARITY_RELATION},
    language::{variable_process, *},
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;

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
            PremiseSource::Task,
            context,
        ),

        // * 📄T="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        // * + B="[aggressive]"
        // * @ C="<<$1 --> [aggressive]> ==> <$1 --> murder>>"
        [SELF, Compound] => compound_and_self(
            cast_compound(belief_term),
            task_term,
            PremiseSource::Belief,
            context,
        ),

        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * + B="{tim}"
        // * @ C="<{tim} --> (/,livingIn,_,{graz})>"
        [SELF, ComponentStatement] => {
            if let Some(belief) = belief {
                detachment(
                    &task_sentence,
                    &belief,
                    PremiseSource::Task,
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
                detachment(
                    &task_sentence,
                    &belief,
                    PremiseSource::Belief,
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
                conditional_deduction_induction(
                    cast_statement(task_term),
                    *b_link.get_index(1).unwrap(),
                    belief_term,
                    &belief,
                    PremiseSource::Task,
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
                conditional_deduction_induction(
                    cast_statement(belief_term),
                    *b_link.get_index(1).unwrap(),
                    task_term,
                    &belief,
                    PremiseSource::Belief,
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
            PremiseSource::Task,
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
                    b_index.unwrap(),
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
            PremiseSource::Belief,
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
                    b_index.expect("B链接索引越界@三段论推理"),
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
                conditional_deduction_induction_with_var(
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
                detachment_with_var(
                    task_sentence,
                    belief,
                    PremiseSource::Task,
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
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // * 🚩合取/析取
    if compound.instanceof_junction() {
        // * 🚩有「当前信念」⇒解构出陈述
        if context.has_current_belief() {
            decompose_statement(compound.get_ref(), &component, compound_from, context);
        }
        // * 🚩否，但包含元素⇒取出词项
        else if compound.get_ref().contain_component(&component) {
            structural_junction(compound.get_ref(), &component, compound_from, context);
        }
    // } else if ((compound instanceof Negation) &&
    // !context.getCurrentTask().isStructural()) {
    }
    // * 🚩否定
    // * 📝【2024-07-22 17:40:06】规则表分派不要过于涉及词项处理：是否要「提取否定内部的词项」要由「具体规则函数」决定
    else if compound.instanceof_negation() {
        transform_negation(compound, compound_from, context)
    }
    // * 🚩其它⇒无结果
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
    use std::cmp::Ordering::*;
    use PremiseSource::*;
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
    compound_from: PremiseSource,
    mut compound: CompoundTerm,
    index: usize,
    mut statement: Statement,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    let component = unwrap_or_return!(?compound.get_ref().component_at(index));
    // ! ⚠️可能与「当前概念」的词项不一致：元素"{tom}"🆚概念"tom"
    let task_is_judgement = context.current_task().get_().is_judgement();
    // * 🚩均为陈述，且为同一类型⇒组合规则
    if component.is_same_type(&statement) {
        // * 其内元素是「合取」且有「当前信念」
        if compound.instanceof_conjunction() && context.has_current_belief() {
            // * 🚩先尝试消去非独变量 #
            // ! ⚠️【2024-08-08 15:56:06】此处不能信任「应用归一化后，再从原来的index处得到元素引用」
            //   * 可能对于「合取」这样的可交换词项，需要重新决定
            let unification_d =
                variable_process::unify_find_d(component, &statement, context.shuffle_rng_seeds());
            // 重新获取一次共同组分
            // * 🚩能消去⇒因变量消元
            if unification_d.has_unification {
                // * 🚩两边同时应用归一化
                let mut component = component.clone();
                unification_d.apply_to(compound.mut_ref(), statement.mut_ref().into_compound_ref());
                unification_d.unify_map_1.apply_to_term(&mut component); // * 📌独立应用一次，应该和compound一样

                // * 🚩现场计算「是否相等」，需要在「变量统一」后执行
                let statement_equals_belief = {
                    // * 🚩复合词项来自任务：`[任务复合, 信念陈述]`；来自信念：`[任务陈述, 信念复合]`
                    let [_, belief_content] = compound_from
                        .select([compound.get_ref().inner, statement.get_ref().statement]);
                    // ? ❓【2024-08-06 20:18:28】是否一定要在此判等？还是直接根据「选中了哪个词项」选择
                    //   * ⚠️注意：即便选中的陈述不是信念，仍有可能「统一后的词项与信念词项相等」
                    *statement.get_ref().statement == *belief_content
                };
                eliminate_var_dep(
                    compound.get_ref(),
                    &component,
                    match statement_equals_belief {
                        true => PremiseSource::Task,
                        false => PremiseSource::Belief,
                    },
                    context,
                );
            }
            // * 🚩不能消去，但任务是判断句⇒内部引入变量
            else if task_is_judgement {
                // && !compound.containComponent(component)) {
                intro_var_inner(
                    statement.get_ref(),
                    component.as_statement().unwrap(),
                    compound.get_ref(),
                    context,
                );
            }
            // * 🚩是疑问句，且能消去查询变量⇒解构出元素作为结论
            else {
                // ! ⚠️【2024-08-08 15:56:06】此处不能信任「应用归一化后，再从原来的index处得到元素引用」
                let unification_q = variable_process::unify_find_q(
                    component,
                    &statement,
                    context.shuffle_rng_seeds(),
                );
                if unification_q.has_unification {
                    // * 🚩两边同时应用归一化
                    let mut component = component.clone();
                    unification_q
                        .apply_to(compound.mut_ref(), statement.mut_ref().into_compound_ref());
                    unification_q.unify_map_1.apply_to_term(&mut component); // * 📌独立应用一次，应该和compound一样
                    // 解构陈述
                    decompose_statement(compound.get_ref(), &component, compound_from, context);
                }
            }
        }
    }
    // if (!task.isStructural() && task.isJudgment()) {
    // * 🚩类型不同 且为双判断⇒结构规则
    else if task_is_judgement {
        let can_compose_both;
        let (compound, statement) = (compound.get_ref(), statement.get_ref());
        // * 🚩涉及的陈述是「继承」
        if statement.instanceof_inheritance() {
            // if (!(compound instanceof SetExt) && !(compound instanceof SetInt)) {
            // * 🚩若能双侧组合⇒双侧组合
            can_compose_both = !(compound.instanceof_set() || compound.instanceof_negation());
            if can_compose_both {
                // {A --> B, A @ (A&C)} |- (A&C) --> (B&C)
                structural_compose_both(compound, index, statement, side, context);
            }
            // * 🚩单侧组合
            structural_compose_one(compound, index, statement, context);
        }
        // * 🚩涉及的陈述是「相似」，但涉及的另一复合词项不是「合取」
        // * 📝「相似」只能双侧组合，可以组合出除「合取」之外的结论
        else if statement.instanceof_similarity() {
            // * 🚩尝试双侧组合
            can_compose_both = !compound.instanceof_conjunction();
            if can_compose_both {
                // {A <-> B, A @ (A&C)} |- (A&C) <-> (B&C)
                structural_compose_both(compound, index, statement, side, context);
            }
        }
    }
}

/// 分派：复合词项与陈述
///
/// # 📄OpenNARS
///
/// Inference between a compound term and a statement
fn component_and_statement(
    compound: CompoundTerm,
    index: usize,
    statement: Statement,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // if (context.getCurrentTask().isStructural()) return;
    match statement.identifier() {
        // * 🚩陈述是「继承」
        INHERITANCE_RELATION => {
            let (compound, statement) = (compound.get_ref(), statement.get_ref());
            // * 🚩集合消去
            structural_decompose_one(compound, index, statement, context);
            // * 🚩尝试两侧都消去：只要不是外延集/内涵集 都可以
            match compound.instanceof_set() {
                // * 🚩集合⇒特殊处理
                // * 📝外延集性质：一元集合⇒最小外延 | 内涵集性质：一元集合⇒最小内涵
                // * <A --> {B}> |- <A <-> {B}>
                true => transform_set_relation(compound, statement, side, context),
                // * 🚩默认⇒两侧消去
                // {(C-B) --> (C-A), A @ (C-A)} |- A --> B
                false => structural_decompose_both(statement, index, context),
            }
        }
        // * 🚩陈述是「相似」⇒总是要两侧消去
        SIMILARITY_RELATION => {
            let (compound, statement) = (compound.get_ref(), statement.get_ref());
            // {(C-B) <-> (C-A), A @ (C-A)} |- A <-> B
            structural_decompose_both(statement, index, context);
            // * 🚩外延集/内涵集⇒尝试转换集合关系
            if compound.instanceof_set() {
                // * 🚩外延集性质：一元集合⇒最小外延 | 内涵集性质：一元集合⇒最小内涵
                // * <A <-> {B}> |- <A --> {B}>
                transform_set_relation(compound, statement, side, context);
            }
        }
        // * 🚩蕴含×否定⇒逆否
        IMPLICATION_RELATION if compound.instanceof_negation() => match index {
            0 => contraposition(statement, PremiseSource::Task, context),
            _ => contraposition(statement, PremiseSource::Belief, context),
        },
        _ => {}
    }
}

/// 分派：复合词项×复合条件
fn compound_and_compound_condition(
    task_sentence: impl Sentence,
    belief: impl Judgement,
    mut task_term: CompoundTerm,
    mut belief_term: Statement,
    b_index: usize,
    context: &mut ReasonContextConcept,
) {
    let rng_seed = context.shuffle_rng_seeds();
    if belief_term.instanceof_implication() {
        // * 🚩尝试统一其中的独立变量
        let can_detach =
            variable_process::unify_find_i(belief_term.get_ref().subject, &task_term, rng_seed)
                .apply_to(
                    belief_term.mut_ref().into_compound_ref(),
                    task_term.mut_ref(),
                );
        match can_detach {
            // * 🚩成功统一 ⇒ 应用「条件分离」规则
            true => detachment_with_var(
                task_sentence,
                belief,
                PremiseSource::Belief,
                SyllogismPosition::from_index(b_index),
                context,
            ),
            // * 🚩未能统一 ⇒ 应用「条件 演绎/归纳」规则
            false => conditional_deduction_induction(
                belief_term,
                b_index, // * 📝Rust允许直接用`as`将枚举转换为数值
                task_term.into(),
                &belief,
                PremiseSource::Belief,
                SyllogismSide::Whole,
                context,
            ),
        }
    }
    // * 🚩此处需要限制「任务词项」是「蕴含」
    else if belief_term.instanceof_equivalence() && task_term.instanceof_implication() {
        // * 🚩条件类比
        conditional_analogy(
            belief_term,
            b_index,
            cast_statement(task_term.into()), // 复合词项强转为陈述
            SyllogismSide::Whole,
            &belief,
            context,
        );
    }
}

/// 🆕匹配分支：复合条件×复合陈述
fn compound_condition_and_compound_statement(
    task_sentence: impl Sentence,
    task_term: Statement,
    t_side: SyllogismPosition,
    belief: impl Judgement,
    belief_term: Statement,
    b_side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    let [task_subject, _] = task_term.sub_pre();
    // * 🚩「否定」⇒继续作为「元素🆚陈述」处理
    if task_subject.instanceof_negation() {
        // 决定参数
        let negation = cast_compound(task_subject.clone());
        let (self_side, statement, statement_side) = match task_sentence.is_judgement() {
            true => (b_side, task_term, t_side),
            _ => (t_side, belief_term, b_side),
        };
        // 统一调用
        component_and_statement(
            negation,
            self_side as usize,
            statement,
            statement_side,
            context,
        );
    }
    // * 🚩一般情况⇒条件演绎/条件归纳
    else {
        // * 📌【2024-08-06 15:53:55】因为是「复合条件×复合陈述」，所以任务句肯定是条件句
        conditional_deduction_induction_with_var(
            PremiseSource::Task,
            task_term,
            t_side as usize,
            belief_term,
            b_side,
            belief,
            context,
        )
    }
}
