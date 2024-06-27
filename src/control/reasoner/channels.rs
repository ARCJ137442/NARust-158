//! 推理器的输入输出（通道）部分

use crate::io::{InputChannel, OutputChannel};
use std::fmt::{Debug, Formatter};

/// 内部的「推理器通道」结构
/// * 🎯在内部实现中分离[推理器](Reasoner)的「输入输出」逻辑
pub(super) struct ReasonerChannels {
    /// 所有输入通道
    pub(super) input_channels: Vec<Box<dyn InputChannel>>,

    /// 所有输出通道
    pub(super) output_channels: Vec<Box<dyn OutputChannel>>,
}

/// 手动实现：输入输出通道 不一定实现了[`Debug`]
impl Debug for ReasonerChannels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasonerChannels")
            .field(
                "input_channels",
                &format!("[Box<dyn InputChannel>; {}]", self.input_channels.len()),
            )
            .field(
                "output_channels",
                &format!("[Box<dyn OutputChannel>; {}]", self.output_channels.len()),
            )
            .finish()
    }
}
