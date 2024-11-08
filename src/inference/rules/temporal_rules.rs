//! 🆕时序规则
//! * 📄参考自ONA的`inference`模块
//! * 🚩其中返回[`Option`]类型的，[`None`]语义均为「修订失败」「证据基重复」等

use crate::{
    control::ReasonContext,
    entity::{Goal, Judgement, Sentence, Stamp, TruthValue},
    global::{Float, OccurrenceTime},
    inference::{Evidential, Truth, TruthFunctions},
    language::Term,
};
use nar_dev_utils::OrSomeRef;
use std::ops::{Add, Div, Mul};

/// * 🚩【2024-09-19 13:26:55】实际上不需要宏，只要解构赋值就行了
/// * 📌【2024-09-19 20:04:53】确实因为「通配」需要一点点，但Rust中用特征就能实现分派
pub fn derivation_stamp(
    a: &impl Evidential,
    b: &impl Evidential,
    context: &impl ReasonContext,
) -> Option<Stamp> {
    let creation_time = a.creation_time().max(b.creation_time());
    Stamp::from_merge(a, b, creation_time, context.max_evidence_base_length())
}

/// 🆕导出的时间和真值
/// * 🚩【2024-09-19 13:26:55】实际上不需要宏，只要解构赋值就行了
pub fn derivation_time(
    a: &impl Sentence,
    b: &impl Sentence,
    context: &impl ReasonContext,
) -> Option<(OccurrenceTime, [TruthValue; 2])> {
    let conclusion_time = b.occurrence_time();
    let truth_a = a.extract_truth_value()?.projection(
        a.occurrence_time(),
        conclusion_time,
        context.truth_projection_decay(),
    );
    let truth_b = b.extract_truth_value()?;
    Some((conclusion_time, [truth_a, truth_b]))
}

/// * 🚩【2024-09-19 13:26:55】实际上不需要宏，只要解构赋值就行了
/// * 🚩【2024-11-08 10:56:05】时间戳的「发生时间」内置在返回的`Stamp`中了
pub fn derivation_stamp_and_time(
    a: &impl Sentence,
    b: &impl Sentence,
    context: &impl ReasonContext,
) -> Option<(Stamp, OccurrenceTime, [TruthValue; 2])> {
    let conclusion_stamp = derivation_stamp(a, b, context)?;
    let (conclusion_time, [truth_a, truth_b]) = derivation_time(a, b, context)?;
    let conclusion = (conclusion_stamp, conclusion_time, [truth_a, truth_b]);
    Some(conclusion)
}

/// 加权平均
/// * 🚩【2024-09-19 13:28:29】泛化此处函数：支持加乘除即可
fn weighted_average<N>(a1: N, a2: N, w1: N, w2: N) -> N
where
    N: Copy + Add<Output = N> + Div<Output = N> + Mul<Output = N>,
{
    (a1 * w1 + a2 * w2) / (w1 + w2)
}

/// {Event a.} |- Event a. Truth_Projection (projecting to current time)
/// * 📌事件更新
/// * 🚩【2024-11-08 12:01:46】返回投影后的时间与真值
///   * ❌暂不使用「复制&修改」与「直接修改语句」的逻辑
///     * 涉及「修改真值」的逻辑需要让「语句」对象可变
///     * 在「特征方法」的语境中较为困难：影响下层几乎所有特征实现
pub fn event_update(
    event: &(impl Sentence + Truth),
    target_time: impl Into<OccurrenceTime>,
    context: &impl ReasonContext,
) -> (OccurrenceTime, TruthValue) {
    let target_time = target_time.into();
    let truth = event.projection(
        event.occurrence_time(),
        target_time,
        context.truth_projection_decay(),
    );
    (target_time, truth)
}

/// {Event a., Event b.} |- Event (a &/ b). Truth_Intersection (after projecting a to b)
/// * 🚩【2024-09-19 19:48:11】无需使用「回写指针」的方式
///   * ✅Rust中可以直接使用[`Option`]同时表示两者
/// * 📌信念相交
///   * 📌将产生「序列」词项
pub fn belief_intersection(
    a: &impl Judgement,
    b: &impl Judgement,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, OccurrenceTime)> {
    assert!(
        b.occurrence_time() >= a.occurrence_time(),
        "after(b,a) violated in Inference_BeliefIntersection"
    );
    let (conclusion_stamp, conclusion_time, [truth_a, truth_b]) =
        derivation_stamp_and_time(a, b, context)?;
    let conclusion_truth = truth_a.intersection(&truth_b);
    let conclusion_term = Term::make_sequence([a.clone_content(), b.clone_content()])?;
    Some((
        conclusion_term,
        conclusion_truth,
        conclusion_stamp,
        conclusion_time,
    ))
}

/// {Event a., Event b.} |- Implication <a =/> b>. Truth_Eternalize(Truth_Induction) (after projecting a to b)
/// * 🚩【2024-09-19 19:51:03】无需使用「回写指针」的方式
///   * ✅Rust中可以直接使用[`Option`]同时表示两者
/// * 📌信念归纳
pub fn belief_induction(
    a: &impl Judgement,
    b: &impl Judgement,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, Float)> {
    assert!(
        b.occurrence_time() >= a.occurrence_time(),
        "after(b,a) violated in Inference_BeliefInduction"
    );
    let (conclusion_stamp, _conclusion_time, [truth_a, truth_b]) =
        derivation_stamp_and_time(a, b, context)?;
    let term = Term::make_temporal_implication(a.clone_content(), b.clone_content())?;
    let truth = truth_b.induction(&truth_a);
    let time_offset = (b.occurrence_time() - a.occurrence_time()).into_float();
    Some((term, truth, conclusion_stamp, time_offset))
}

/// {Implication <a =/> b>., <a =/> b>.} |- Implication <a =/> b>. Truth_Revision
/// * 📌蕴含修订
/// * 🚩词项相同，真值修订，时间偏移取加权平均
pub fn implication_revision(
    a: &impl Judgement,
    b: &impl Judgement,
    context: &impl ReasonContext,
) -> Option<(TruthValue, Stamp, Float)> {
    let conclusion_stamp = derivation_stamp(a, b, context)?; // ! 🕒【2024-09-19 19:59:15】18小时前ONA有更新
    let occurrence_time_offset_avg = weighted_average(
        a.occurrence_time_offset(),
        b.occurrence_time_offset(),
        a.confidence().c2w(),
        b.confidence().c2w(),
    );
    let truth = a.revision(b); // ! 🕒【2024-09-19 19:59:15】18小时前ONA有更新
    Some((truth, conclusion_stamp, occurrence_time_offset_avg))
}

/// {Event b!, Implication <a =/> b>.} |- Event a! Truth_Deduction
pub fn goal_deduction(
    component: &impl Goal,
    implication: &impl Judgement,
    current_time: impl Into<OccurrenceTime>,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, OccurrenceTime)> {
    debug_assert!(
        implication.content().instanceof_implication()
            || implication.content().instanceof_temporal_implication(),
        "Not a valid implication term!"
    );
    let conclusion_stamp = derivation_stamp(component, implication, context)?;
    let precondition = implication
        .content()
        .as_statement()
        .unwrap()
        .subject
        .as_compound()?;
    // extract precondition: (plus unification once vars are there)
    let term = precondition.precondition_without_op().clone();
    let truth = component.goal_deduction(implication);
    let occurrence_time = current_time.into();
    Some((term, truth, conclusion_stamp, occurrence_time))
}

/// {Event (a &/ b)!, Event a.} |- Event b! Truth_Deduction
pub fn goal_sequence_deduction(
    component: &impl Judgement,
    compound: &impl Goal,
    current_time: impl Into<OccurrenceTime>,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, OccurrenceTime)> {
    let current_time = current_time.into();
    let conclusion_stamp = derivation_stamp(compound, component, context)?;
    let (_, truth_compound_updated) = event_update(component, current_time, context);
    let (_, truth_component_updated) = event_update(compound, current_time, context);
    let term = component.clone_content();
    let truth = truth_compound_updated.goal_deduction(&truth_component_updated);
    Some((term, truth, conclusion_stamp, current_time))
}

/// {Event a!, Event a!} |- Event a! Truth_Revision or Choice (dependent on evidential overlap)
/// * 🚩【2024-09-29 20:13:49】现在返回(新事件, 是否已被修订)
/// * ️📝修订结果有三种情况：
///   * 无修订：返回传入的事件本身
///   * 更新事件：返回更新后的传入事件
///   * 成功修订：返回已存在事件与传入事件合并后的新事件
/// * 📌修订结果的标点与传入的事件一致
pub fn revision_and_choice<S: Sentence + Truth>(
    existing_potential: impl OrSomeRef<S>,
    incoming_spike: &S,
    context: &impl ReasonContext, // * 🚩【2024-11-08 20:35:34】ONA的「当前时间」替换为上下文
) -> Option<((Term, TruthValue, Stamp, OccurrenceTime), bool)> {
    let mut revised = false;
    let copied_event = |event: &S| {
        (
            event.clone_content(),
            TruthValue::from(event),
            Stamp::with_old(event, event.creation_time()),
            event.occurrence_time(),
        )
    };
    let Some(existing_potential) = existing_potential.or_some_ref() else {
        return Some((copied_event(incoming_spike), revised));
    };

    let later_occurrence =
        (existing_potential.occurrence_time()).max(incoming_spike.occurrence_time());
    let (_, existing_updated) = event_update(existing_potential, later_occurrence, context);
    let (_, incoming_updated) = event_update(incoming_spike, later_occurrence, context);
    //check if there is evidential overlap
    let overlap = incoming_spike.evidential_overlap(existing_potential);
    let is_dep_var_conj = (incoming_spike.content().instanceof_conjunction()
        || incoming_spike.content().instanceof_sequence())
        && incoming_spike.content().contain_var_d();
    //if there is or the terms aren't equal, apply choice, keeping the stronger one:
    if overlap
        || is_dep_var_conj
        || existing_potential.occurrence_time().not_eternal()
            && existing_potential.occurrence_time() != incoming_spike.occurrence_time()
        || existing_potential.content() != incoming_spike.content()
    {
        // 用「更新后更强信念的事件」替代已存在的事件
        if existing_updated.confidence() >= incoming_updated.confidence() {
            Some((copied_event(existing_potential), revised))
        } else {
            Some((copied_event(incoming_spike), revised)) //preserves timing of incoming
        }
    } else {
        //and else revise, increasing the "activation potential"
        // * 🚩【2024-11-08 20:58:31】因「事件更新」无法返回整个语句（根源：没有好的「拷贝语句」方式），
        //   * 此处采取「修订旧事件再取代真值」的方法
        let revised_spike = incoming_updated.revision(&existing_updated);
        let (term, _truth, stamp, occurrence_time) =
            event_revision(existing_potential, incoming_spike, context)?;
        assert!(
            revised_spike.confidence() >= existing_updated.confidence(),
            "Revision outcome can't be lower in confidence than existing event",
        );
        revised = true;
        let event = (term, revised_spike, stamp, occurrence_time);
        Some((event, revised))
    }
}

/// {Event a., Implication <a =/> b>.} |- Event b.  Truth_Deduction
pub fn belief_deduction(
    component: &impl Judgement,
    compound: &impl Judgement,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, OccurrenceTime)> {
    // ! 🕒【2024-09-19 20:55:05】↓22小时前ONA有更新
    debug_assert!(
        compound.content().instanceof_implication()
            || compound.content().instanceof_temporal_implication(),
        "Not a valid implication term!"
    );
    let conclusion_stamp = derivation_stamp(component, compound, context)?;
    let postcondition = compound.content().as_statement()?.predicate.clone();
    let truth = compound.deduction(component);
    let occurrence_time = component.occurrence_time() + compound.occurrence_time_offset();
    Some((postcondition, truth, conclusion_stamp, occurrence_time))
}

/// 事件修订
/// * 📌修订结果的标点与传入的事件一致
pub fn event_revision<S: Sentence + Truth>(
    a: &S,
    b: &S,
    context: &impl ReasonContext,
) -> Option<(Term, TruthValue, Stamp, OccurrenceTime)> {
    let (conclusion_stamp, conclusion_time, [truth_a, truth_b]) =
        derivation_stamp_and_time(a, b, context)?;
    let term = a.clone_content();
    let truth = truth_a.revision(&truth_b);
    Some((term, truth, conclusion_stamp, conclusion_time))
}
