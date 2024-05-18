//! 🆕有关「推理器报告」或「推理器记录」
//! * 🎯承载原`Memory.report`、`Memory.exportStrings`逻辑
//! * 🎯推理器（原记忆区）输出信息
//! * 🚩【2024-05-06 09:35:37】复用[`navm`]中的「NAVM输出」
use navm::output::Output;
use std::collections::VecDeque;

/// 模拟`Memory.exportStrings`、`nars.inference.IInferenceRecorder`
/// * 🎯推理记忆者，负责在推理器中记录「导出结论」「执行操作」等输出信息
/// * 🚩【2024-05-18 12:02:49】目前从「记忆区」中独立而来（作为全局对象）
///
/// # 📄OpenNARS
///
/// ## `Memory.exportStrings`
///
/// List of Strings or Tasks to be sent to the output channels
///
/// ## `nars.inference.IInferenceRecorder`
///
/// 🈚
pub trait ReasonRecorder {
    /// 缓存的输出缓冲区
    /// * 🚩【2024-05-07 20:09:49】目前使用[`VecDeque`]队列实现
    fn cached_outputs(&self) -> &VecDeque<Output>;
    /// [`MemoryRecorder::cached_outputs`]的可变版本
    fn __cached_outputs_mut(&mut self) -> &mut VecDeque<Output>;

    /// 长度大小
    #[inline]
    fn len_output(&self) -> usize {
        self.cached_outputs().len()
    }

    /// 是否为空
    #[inline]
    fn no_output(&self) -> bool {
        self.cached_outputs().is_empty()
    }

    /// 置入NAVM输出（在末尾）
    #[inline]
    fn put(&mut self, output: Output) {
        self.__cached_outputs_mut().push_back(output)
    }

    /// 取出NAVM输出（在开头）
    /// * ⚠️可能没有（空缓冲区）
    #[inline]
    fn take(&mut self) -> Option<Output> {
        self.__cached_outputs_mut().pop_front()
    }

    /// 清空
    /// * 🎯用于推理器「向外输出并清空内部结果」备用
    ///   * 🚩【2024-05-13 02:13:21】现在直接用`while let Some(output) = self.take()`型语法
    #[inline]
    fn clear(&mut self) {
        self.__cached_outputs_mut().clear()
    }
}

/// 🆕[`MemoryRecorder`]的具体特征
/// * ✅统一的构造函数
pub trait MemoryRecorderConcrete: ReasonRecorder + Sized {
    /// 🆕构造函数
    /// * 🚩构造一个空的「记忆区记录者」
    fn new() -> Self;
}

/// 「记忆区记录器」初代实现
/// * 🚩使用「NAVM输出」表示
#[derive(Debug, Clone, Default)]
pub struct MemoryRecorderV1 {
    /// 输出缓冲区
    cached_outputs: VecDeque<Output>,
}

/// 实现「记忆区记录器」（字段对应）
impl ReasonRecorder for MemoryRecorderV1 {
    fn cached_outputs(&self) -> &VecDeque<Output> {
        &self.cached_outputs
    }

    fn __cached_outputs_mut(&mut self) -> &mut VecDeque<Output> {
        &mut self.cached_outputs
    }
}

impl MemoryRecorderConcrete for MemoryRecorderV1 {
    // 构造函数
    // * 🚩默认构造空数组
    #[inline]
    fn new() -> Self {
        Self::default()
    }
}
