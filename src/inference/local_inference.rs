//! 本地推理
//! * 🎯承载原先「直接推理」的部分
//! * 📝其中包含「修订规则」等

use crate::{
    control::ReasonContextDirect,
    entity::{Punctuation, Sentence},
    util::RefCount,
};

/// 本地推理 入口函数
pub fn process_direct(context: &mut ReasonContextDirect) {
    // * 🚩先根据类型分派推理
    let task_punctuation = context.current_task.get_().punctuation();

    use Punctuation::*;
    match task_punctuation {
        Judgement => process_judgement(context),
        Question => process_question(context),
    }
}

fn process_judgement(context: &mut ReasonContextDirect) {
    todo!()
}

fn process_question(context: &mut ReasonContextDirect) {
    todo!()
}
