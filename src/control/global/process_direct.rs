//! 推理器有关「直接推理/立即推理」的功能
//! * 🎯模拟以`Memory.immediateProcess`为入口的「直接推理」
//! * 🎯将其中有关「直接推理」的代码摘录出来
//!   * 📌处理新任务(内部) from 工作周期(@记忆区)
//!   * 📌处理新近任务(内部) from 工作周期(@记忆区)
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部 @概念) from 直接处理
//!   * 📌处理问题(内部 @概念) from 直接处理
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! ## 🚩【2024-05-18 14:48:57】有关「复制以防止借用问题」的几个原则
//!
//! * 📌从「词项」到「语句」均为「可复制」的，但只应在「不复制会导致借用问题」时复制
//! * 📌「任务」「概念」一般不应被复制
//! * 📌要被修改的对象**不应**被复制：OpenNARS将修改这些量，以便在后续被使用

use crate::{control::*, entity::*, inference::*, nars::*, storage::*, types::TypeContext, *};
use navm::output::Output;

/// 推理器与「工作周期」有关的功能
pub trait ReasonerDirectProcess<C: TypeContext>: Reasoner<C> {
    // TODO: 对标改版之`ProcessDirect.java`，重做
}

/// 通过「批量实现」自动加功能
impl<C: TypeContext, T: Reasoner<C>> ReasonerDirectProcess<C> for T {}
