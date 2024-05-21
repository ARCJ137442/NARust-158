//! 基于「推理器」「推理上下文」有关「推理周期」的操作
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现OpenNARS
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌吸收推理上下文(新)
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️

use super::*;
use crate::{entity::*, inference::*, nars::*, types::TypeContext};
use nar_dev_utils::list;
use navm::output::Output;

// TODO: 是否考虑「推理器」并吞「记忆区」
// * 💡如：就将「记忆区」变成一个纯粹的「增强版概念袋」使用

/// 🆕推理器「概念推理」的结果
pub enum DirectProcessResult<D, R, T>
where
    D: TypeContext,
    R: TypeContext,
    T: TermLink,
{
    /// * 📌情况1：「直接推理」完毕，但因各种情况没打算开始「概念推理」
    ///   * 📄如：先前的「直接推理」已有结果，不太需要「概念推理」
    ///   * 📄如：未能选中「当前概念」「当前任务链」
    ///   * 📄如：选中的「任务链」类型是「转换」，只需应用NAL-4相关规则
    OnlyDirect(D),
    /// * 📌情况2：「直接推理」之后，已构建好「推理上下文」并可以开始「概念推理」
    ///   * ⚠️此时「当前概念」「当前任务链」「当前任务」均已准备好
    ///   * ✨亦包含【将要参与轮流`reason`】的「词项链列表」
    ContextReady(R, Vec<T>),
}

/// 推理器与「工作周期」有关的功能
pub trait ReasonerWorkCycle<C: TypeContext>: Reasoner<C> {
    // TODO: 对标改版之`Memory.workCycle`，重做
}

/// 通过「批量实现」自动加功能
impl<C: TypeContext, T: Reasoner<C>> ReasonerWorkCycle<C> for T {}
