//! 🆕时序规则
//! * 📄参考自ONA的`inference`模块

use crate::{
    control::ReasonContext,
    entity::{Sentence, Stamp, TruthValue},
    global::OccurrenceTime,
    inference::{Evidential, TruthFunctions},
};

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

// TODO: 具体真值函数
