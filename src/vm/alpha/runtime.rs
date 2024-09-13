//! 虚拟机运行时
//!
//! * ✅【2024-05-15 16:57:37】初代全功能实现

use super::{ChannelIn, ChannelOut, ReasonerChannels};
use crate::{control::Reasoner, global::RC, inference::InferenceEngine, parameters::Parameters};
use anyhow::Result;
use nar_dev_utils::RefCount;
use navm::{
    cmd::Cmd,
    output::Output,
    vm::{VmRuntime, VmStatus},
};

/// 虚拟机运行时
/// * 🎯包装一个虚拟机，以跳出孤儿规则的限制
#[derive(Debug)]
pub struct RuntimeAlpha {
    /// IO通道
    pub(super) io_channels: ReasonerChannels,
    /// 内部推理器字段
    pub(super) reasoner: Reasoner,
    /// 输入通道的共享引用
    i_channel: RC<ChannelIn>,
    /// 输出通道的共享引用
    /// * 🎯避免「运行时→推理器→通道→运行时」的循环引用
    /// * 🚩「缓存的输出」亦包含在内
    o_channel: RC<ChannelOut>,
}

/// 自身实现
impl RuntimeAlpha {
    /// 构造函数
    /// * 🚩【2024-05-15 10:40:49】暂不允许「直接由推理器创建」
    ///   * 📌需要更精细地控制「内部推理器」的状态与成员
    /// * 🚩【2024-06-28 22:54:15】现在需要传递推理引擎
    /// * 🚩【2024-06-29 00:59:24】现在需要给出「输入源」（当输入），亦可不
    pub fn new(
        name: impl Into<String>,
        hyper_parameters: Parameters,
        inference_engine: InferenceEngine,
    ) -> Self {
        // * 🚩创建推理器
        let reasoner = Reasoner::new(name.into(), hyper_parameters, inference_engine);

        // * 🚩创建并加入通道
        let (io_channels, i_channel, o_channel) = default_channels();

        // * 🚩构造自身
        Self {
            // * 🚩载入推理器
            reasoner,
            // 通道结构
            io_channels,
            // * 🚩空通道
            i_channel,
            // * 🚩空通道
            o_channel,
        }
    }
}

fn default_channels() -> (ReasonerChannels, RC<ChannelIn>, RC<ChannelOut>) {
    let mut io_channels = ReasonerChannels::new();

    let i_channel = RC::new_(ChannelIn::new());

    io_channels.add_input_channel(Box::new(i_channel.clone()));
    // * ✅解决：在「推理器」中细化生命周期约束，现在不再报错与要求`'static`

    let o_channel = RC::new_(ChannelOut::new());
    io_channels.add_output_channel(Box::new(o_channel.clone()));
    // * ✅解决：在「推理器」中细化生命周期约束，现在不再报错与要求`'static`

    (io_channels, i_channel, o_channel)
}

/// 实现[虚拟机运行时](VmRuntime)
impl VmRuntime for RuntimeAlpha {
    fn input_cmd(&mut self, cmd: Cmd) -> Result<()> {
        // ! ⚠️不要直接朝推理器输入NAVM指令，要利用推理器自身的通道机制
        // * 🚩将指令置入通道中
        self.i_channel.mut_().put(cmd);
        // * 🚩让推理器处理一次完整输入输出
        // * 📌其中包括`NSE`指令，会将执行的回执（输出）单独带出
        self.handle_io();
        Ok(())
    }

    fn fetch_output(&mut self) -> Result<Output> {
        self.o_channel
            .mut_()
            .fetch()
            .ok_or(anyhow::anyhow!("没有输出"))
    }

    fn try_fetch_output(&mut self) -> Result<Option<Output>> {
        Ok(self.o_channel.mut_().fetch())
    }

    fn status(&self) -> &VmStatus {
        // * 🚩【2024-05-15 16:39:12】始终在运行
        // * ❓貌似Rust版本并不一定要像Java版本那样区分「在运行」与「不在运行」——随时输入随时处理
        &VmStatus::Running
    }

    fn terminate(&mut self) -> Result<()> {
        // * 🚩重置推理器
        self.reasoner.reset();
        // * 🚩返回
        Ok(())
    }
}
