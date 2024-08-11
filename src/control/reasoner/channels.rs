//! 推理器的输入输出（通道）部分

use super::Reasoner;
use crate::io::{InputChannel, OutputChannel};
use std::fmt::{Debug, Formatter};

/// 输入通道对象
pub(in super::super) type InputChannelObj = Box<dyn InputChannel>;

/// 输出通道对象
pub(in super::super) type OutputChannelObj = Box<dyn OutputChannel>;

/// 内部的「推理器通道」结构
/// * 🎯在内部实现中分离[推理器](Reasoner)的「输入输出」逻辑
///
/// * 🚩【2024-08-12 00:11:05】暂且搁置对「通道」的序列反序列化尝试
///   * 💭函数指针都够呛，特征对象就更难被序列化了……
#[derive(Default)]
pub(in super::super) struct ReasonerChannels {
    /// 所有输入通道
    pub input_channels: Vec<InputChannelObj>,

    /// 所有输出通道
    pub output_channels: Vec<OutputChannelObj>,
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

/// 为「推理器」扩展方法
impl Reasoner {
    /* 通道相关 */

    /// 模拟`ReasonerBatch.addInputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn add_input_channel(&mut self, channel: InputChannelObj) {
        self.io_channels.input_channels.push(channel);
    }

    /// 模拟`ReasonerBatch.addOutputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn add_output_channel(&mut self, channel: OutputChannelObj) {
        self.io_channels.output_channels.push(channel);
    }

    // ! ❌不模拟`ReasonerBatch.removeInputChannel`
    //   * 📝OpenNARS中仅用于「请求推理器移除自身」
    //   * 🚩这实际上可以被「标记『待移除』，下次遍历到时直接删除」的方法替代
    //   * ✅同时避免了「循环引用」「动态判等」问题

    // ! ❌不模拟`ReasonerBatch.removeOutputChannel`
}
