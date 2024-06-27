//! 有关「推理器报告」或「推理器记录」
//! * 🎯承载原`Memory.report`、`Memory.exportStrings`逻辑
//! * 🎯推理器（原记忆区）输出信息
//! * 🚩【2024-05-06 09:35:37】复用[`navm`]中的「NAVM输出」

use navm::output::Output;
use std::collections::VecDeque;

use super::Reasoner;

#[derive(Debug, Clone, Default)]
pub(in super::super) struct ReasonRecorder {
    /// 缓存的NAVM输出
    cached_outputs: VecDeque<Output>,
}

impl ReasonRecorder {
    // /// 长度大小
    // pub fn len_output(&self) -> usize {
    //     self.cached_outputs.len()
    // }

    // /// 是否为空
    // pub fn no_output(&self) -> bool {
    //     self.cached_outputs.is_empty()
    // }

    /// 置入NAVM输出（在末尾）
    pub fn put(&mut self, output: Output) {
        self.cached_outputs.push_back(output)
    }

    /// 取出NAVM输出（在开头）
    /// * ⚠️可能没有（空缓冲区）
    pub fn take(&mut self) -> Option<Output> {
        self.cached_outputs.pop_front()
    }

    /// 清空
    /// * 🎯用于推理器「向外输出并清空内部结果」备用
    ///   * 🚩【2024-05-13 02:13:21】现在直接用`while let Some(output) = self.take()`型语法
    pub fn reset(&mut self) {
        self.cached_outputs.clear()
    }
}

/// 为「推理器」扩展方法
impl Reasoner {
    /// 报告输出
    pub fn report(&mut self, output: Output) {
        self.recorder.put(output);
    }
}
