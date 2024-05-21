//! 推理器有关「概念推理/高级推理」的功能
//! * 🎯模拟以`RuleTables.reason`为入口的「概念推理」
//!   * 📌处理概念(内部) from 工作周期
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-05-18 16:36:06】目前从「推理周期」迁移出来

use super::*;
use crate::{entity::*, inference::*, nars::*, storage::*, types::TypeContext, *};
use navm::output::Output;

/// 推理器与「概念推理」有关的功能
pub trait ReasonerConceptProcess<C: TypeContext>: Reasoner<C> {
    // TODO: 对标改版之`ProcessReason.java`，重做
}

/// 通过「批量实现」自动加功能
impl<C: TypeContext, T: Reasoner<C>> ReasonerConceptProcess<C> for T {}
