//! 🎯复刻OpenNARS `nars.inference.CompositionalRules`
//!
//! * ✅【2024-05-12 00:47:43】初步复现方法API
//! * ♻️【2024-08-05 17:31:37】开始根据改版OpenNARS重写

use super::SyllogismPosition;
use crate::{
    control::*,
    entity::*,
    inference::{rules::utils::*, *},
    io::symbols::*,
    language::*,
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;
use ReasonDirection::*;
use SyllogismPosition::*;

/* -------------------- intersections and differences -------------------- */

/// 🆕作为「集合」操作：交并差
pub fn compose_as_set(
    task_content: StatementRef,
    shared_term_i: SyllogismPosition,
    component_common: &Term,
    component_t: &Term,
    component_b: &Term,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// * 📌根据主谓项、真值 创建新内容，并导出结论
///
/// # 📄OpenNARS
///
/// Finish composing implication term
fn process_composed(
    task_content: Statement,
    subject: Term,
    predicate: Term,
    truth: TruthValue,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// # 📄OpenNARS
///
/// ```nal
/// {<(S|P) ==> M>, <P ==> M>} |- <S ==> M>
/// ```
pub fn decompose_compound(
    compound: CompoundTermRef,
    component: &Term,
    component_common: &Term,
    side: SyllogismPosition,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// # 📄OpenNARS
///
/// ```nal
/// {(||, S, P), P} |- S
// {(&&, S, P), P} |- S
/// ```
pub fn decompose_statement(
    compound: CompoundTermRef,
    component: &Term,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/* --------------- rules used for variable introduction --------------- */

/// 🆕入口之一：变量引入
/// ! ⚠️【2024-07-23 12:20:18】逻辑未完全被测试覆盖，代码理解度低
/// * 📝【2024-07-23 12:04:33】OpenNARS 3.1.0仍然没有样例注释……
/// * 📄一例（平凡情况）：
///   * originalMainSentence = "<<$1 --> swimmer> ==> <$1 --> bird>>"
///   * subSentence = "<bird --> animal>"
///   * component = "<$1 --> bird>"
///   * subContent = "<bird --> animal>"
///   * index = 1 @ originalMainSentence
///   * => "<<$1 --> swimmer> ==> <$1 --> bird>>"
pub fn intro_var_same_subject_or_predicate(
    original_main_sentence: &impl Judgement,
    sub_sentence: &impl Judgement,
    component: &Term,
    sub_content: CompoundTermRef,
    side: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// Introduce a dependent variable in an outer-layer conjunction
/// * 📝「变量外引入」系列规则
///
/// * 📌导出结论：「正反似合」
///   * 外延正传递（归因/归纳）
///   * 外延反传递（归因/归纳）
///   * 相似の传递（比较）
///   * 因变量引入（合取）
///
/// * 📄@主项: "<M --> S>" × "<M --> P>"
///   * => "<<$1 --> S> ==> <$1 --> P>>"
///   * => "<<$1 --> P> ==> <$1 --> S>>"
///   * => "<<$1 --> S> <=> <$1 --> P>>"
///   * => "(&&,<#1 --> S>,<#1 --> P>)"
///
/// * 📄@谓项: "<S --> M>" × "<P --> M>"
///   * => "<<S --> $1> ==> <P --> $1>>"
///   * => "<<P --> $1> ==> <S --> $1>>"
///   * => "<<P --> $1> <=> <S --> $1>>"
///   * => "(&&,<P --> #1>,<S --> #1>)"
pub fn intro_var_outer(
    task_content: StatementRef,
    belief_content: StatementRef,
    shared_term_i: SyllogismPosition,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_ind(
    task_content: Statement,
    belief_content: Statement,
    side: SyllogismPosition,
) -> [Term; 2] {
    todo!()
}

/// 🆕以「变量外引入」的内部词项，计算「引入状态」陈述
/// * 📌引入的是「独立变量/自变量」"$"
/// * 🎯产生的陈述（二元组）用于生成新结论内容
fn intro_var_states_dep(
    task_content: Statement,
    belief_content: Statement,
    side: SyllogismPosition,
) -> [Term; 2] {
    todo!()
}

/// 「变量外引入」规则 结论1
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> animal> ==> <$1 --> swimmer>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<sport --> $1> ==> <chess --> $1>>"
fn intro_var_outer1(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论2
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> swimmer> ==> <$1 --> animal>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<chess --> $1> ==> <sport --> $1>>"
fn intro_var_outer2(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论3
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "<<$1 --> animal> <=> <$1 --> swimmer>>"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "<<chess --> $1> <=> <sport --> $1>>"
fn intro_var_outer3(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量外引入」规则 结论4
/// * 📄"<bird --> animal>" × "<bird --> swimmer>"
///   * => "(&&,<#1 --> animal>,<#1 --> swimmer>)"
/// * 📄"<sport --> competition>" × "<chess --> competition>"
///   * => "(&&,<chess --> #1>,<sport --> #1>)"
fn intro_var_outer4(
    state_1: Term,
    state_2: Term,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// * 📝入口2：变量内引入
///
/// # 📄OpenNARS
///
/// ```nal
/// {<M --> S>, <C ==> <M --> P>>} |- <(&&, <#x --> S>, C) ==> <#x --> P>>
/// {<M --> S>, (&&, C, <M --> P>)} |- (&&, C, <<#x --> S> ==> <#x --> P>>)
/// ```
pub fn intro_var_inner(
    premise_1: StatementRef,
    premise_2: StatementRef,
    old_compound: CompoundTermRef,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 🆕以「变量内引入」的内部词项，计算「共有词项」
/// * 🎯产生的词项（二元组/空）用于生成新结论内容
fn intro_var_commons(premise_1: Statement, premise_2: Statement) -> [Term; 2] {
    todo!()
}

/// 「变量内引入」规则 结论1
/// * 📝引入第二个变量，并在替换后产生一个合取
///
/// * 📄"<{lock1} --> lock>" × "<{lock1} --> (/,open,$1,_)>"
/// * * @ "<<$1 --> key> ==> <{lock1} --> (/,open,$1,_)>>"
/// * * => "(&&,<#2 --> lock>,<<$1 --> key> ==> <#2 --> (/,open,$1,_)>>)"
///
/// * 📄"<{Tweety} --> [chirping]>" × "<robin --> [chirping]>"
/// * * @ "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)"
/// * * => "(&&,<robin --> #1>,<robin --> [with_wings]>,<{Tweety} --> #1>)"
fn intro_var_inner1(
    premise_1: Statement,
    old_compound: CompoundTerm,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    common_term_1: Term,
    common_term_2: Term,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// 「变量内引入」规则 结论2
/// * 📝引入第二个变量，并在替换后产生一个蕴含
///
/// * 📄"<{lock1} --> lock>" × "<{lock1} --> (/,open,$1,_)>"
/// * * @ "<<$1 --> key> ==> <{lock1} --> (/,open,$1,_)>>"
/// * * => "<(&&,<$1 --> key>,<$2 --> lock>) ==> <$2 --> (/,open,$1,_)>>"
///
/// * 📄"<{Tweety} --> [chirping]>" × "<robin --> [chirping]>"
/// * * @ "(&&,<robin --> [chirping]>,<robin --> [with_wings]>)"
/// * * => "<<{Tweety} --> $1> ==> (&&,<robin --> $1>,<robin --> [with_wings]>)>"
fn intro_var_inner2(
    premise_1: Statement,
    old_compound: CompoundTerm,
    truth_t: &impl Truth,
    truth_b: &impl Truth,
    common_term_1: Term,
    common_term_2: Term,
    context: &mut ReasonContextConcept,
) {
    // TODO
}

/// # 📄OpenNARS
///
/// Introduce a second independent variable into two terms with a common
/// component
fn second_common_term([term1, term2]: [&Term; 2], side: SyllogismPosition) -> &Term {
    todo!()
}

/// 因变量消元
/// * 📝用于处理类似「存在变量」的情况
///
/// # 📄OpenNARS
///
/// ```nal
/// {(&&, <#x() --> S>, <#x() --> P>), <M --> P>} |- <M --> S>
/// ```
pub fn eliminate_var_dep(
    compound: CompoundTermRef,
    component: &Term,
    compound_from: PremiseSource,
    context: &mut ReasonContextConcept,
) {
    // TODO
}
