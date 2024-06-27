//! 推理器有关「概念推理/高级推理」的功能
//! * 🎯模拟以`RuleTables.reason`为入口的「概念推理」
//!   * 📌处理概念(内部) from 工作周期
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! ## Logs
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-05-18 16:36:06】目前从「推理周期」迁移出来
//! * ♻️【2024-06-26 11:59:58】开始根据改版OpenNARS重写

use crate::{
    control::{ReasonContextConcept, Reasoner},
    inference::InferenceEngine,
};
use nar_dev_utils::unwrap_or_return;

impl Reasoner {
    /// 概念推理
    /// * 📌「概念推理」控制机制的入口函数
    pub(in crate::control) fn process_reason(&mut self) {
        // * 🚩从「直接推理」到「概念推理」过渡 阶段 * //
        // * 🚩选择概念、选择任务链、选择词项链（中间亦有推理）⇒构建「概念推理上下文」
        let (context, inference_engine) = unwrap_or_return!(?self.preprocess_concept() => ());
        // * 🚩内部概念高级推理 阶段 * //
        // * 🚩【2024-06-27 21:37:10】此处内联整个函数，以避免借用问题
        Self::process_concept(context, inference_engine);
    }

    /// * 🚩因返回后的值可能借用着`self`，因此此处需要「抽取字段」赖返回更多值
    ///   * 🎯避免「返回之后的借用问题」
    fn preprocess_concept(&mut self) -> Option<(ReasonContextConcept, &mut dyn InferenceEngine)> {
        Some((todo!(), &mut *self.inference_engine))
    }

    /// 具体形式有待商议（借用问题）
    fn process_concept(
        mut context: ReasonContextConcept,
        inference_engine: &mut dyn InferenceEngine,
    ) {
        // * 🚩推理引擎推理
        inference_engine.reason(&mut context);
        todo!()
    }
}
