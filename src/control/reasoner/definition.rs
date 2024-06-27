//! 推理器 定义
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:02:36】开始根据改版OpenNARS重写

use std::fmt::{Debug, Formatter};

use super::{ReasonRecorder, ReasonerChannels};
use crate::{
    control::Parameters, entity::RCTask, global::ClockTime, inference::InferenceEngine,
    storage::Memory,
};
use navm::output::Output;

// ! ❌【2024-06-27 18:01:23】不复刻静态常量`Reasoner.DEBUG`

/// 作为结构体的「推理器」
#[derive(Debug)]
pub struct Reasoner {
    /// 推理器「名称」
    name: String,

    /// 超参数
    /// * 📌【2024-06-26 23:55:40】需要部分公开，以便在其它地方解决「借用冲突」问题
    pub(in crate::control) parameters: Parameters,

    /// 记忆区
    pub(in crate::control) memory: Memory,

    /// 记录器
    recorder: ReasonRecorder,

    /// IO通道
    io_channels: ReasonerChannels,

    /// 系统时钟
    clock: ClockTime,

    /// 状态「运行中」
    running: bool,

    /// 剩下的用于「步进」的步数
    /// * 💭最初用于多线程，但目前的NARust中拟采用单线程
    ///
    /// TODO: ❓明确「是否需要」
    walking_steps: usize,

    /// 决定是否「完成了输入」
    finished_inputs: bool,

    /// 最后一个输出之前的步数
    timer: usize,

    /// 静默等级（0~100）
    /// * 🚩【2024-06-27 19:06:32】不同于OpenNARS，此处仅使用普通整数
    silence_value: usize,

    /// 时间戳序列号（递增序列号）
    stamp_current_serial: ClockTime,

    /// 使用的推理引擎
    inference_engine: Box<dyn InferenceEngine>,
}

/// 为动态的
impl Debug for dyn InferenceEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InferenceEngine")
            .field("..", &"..")
            .finish()
    }
}

impl Reasoner {
    pub fn parameters(&self) -> &Parameters {
        todo!()
    }

    pub fn silence_value(&self) -> usize {
        todo!()
    }

    pub fn time(&self) -> ClockTime {
        todo!()
    }

    pub fn memory(&self) -> &Memory {
        todo!()
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        todo!()
    }

    pub fn add_new_task(&mut self, task_rc: RCTask) {
        todo!()
    }

    pub fn report(&mut self, output: Output) {
        todo!()
    }
}
