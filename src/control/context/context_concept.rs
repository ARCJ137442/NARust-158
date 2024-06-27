//! 「概念推理上下文」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 23:49:25】开始根据改版OpenNARS重写

use super::ReasonContextCore;
use crate::{entity::TaskLink, storage::Memory};

/// 概念推理上下文
#[derive(Debug)]
pub struct ReasonContextConcept<'this> {
    /// 内部存储的「上下文核心」
    core: ReasonContextCore<'this>,

    /// 对「记忆区」的反向引用
    memory: &'this Memory,

    /// 选中的任务链
    /// * 📌【2024-05-21 20:26:30】不可空！
    /// * 📌构造后不重新赋值，但内部可变（预算推理/反馈预算值）
    current_task_link: TaskLink,

    // TODO
}
