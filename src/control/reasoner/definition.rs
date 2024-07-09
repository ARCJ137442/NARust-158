//! 推理器 定义
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:02:36】开始根据改版OpenNARS重写

use super::{ReasonRecorder, ReasonerChannels, ReasonerDerivationData};
use crate::{control::Parameters, global::ClockTime, inference::InferenceEngine, storage::Memory};
use navm::output::Output;
use rand::{rngs::StdRng, SeedableRng};
use std::fmt::Debug;

// ! ❌【2024-06-27 18:01:23】不复刻静态常量`Reasoner.DEBUG`

/// 作为结构体的「推理器」
#[derive(Debug)]
pub struct Reasoner {
    /// 推理器「名称」
    name: String,

    /// 超参数
    /// * 📌【2024-06-26 23:55:40】需要部分公开，以便在其它地方解决「借用冲突」问题
    pub(in super::super) parameters: Parameters,

    /// 记忆区
    pub(in super::super) memory: Memory,

    /// 记录器
    pub(super) recorder: ReasonRecorder,

    /// IO通道
    pub(in super::super) io_channels: ReasonerChannels,

    /// 使用的推理引擎
    pub(in super::super) inference_engine: InferenceEngine,

    /// 推理过程的「中间数据」
    pub(in super::super) derivation_datas: ReasonerDerivationData,

    /// 系统时钟
    pub(in super::super) clock: ClockTime,

    // ! ❌不再状态「运行中」，因为NARust-158是始终运行的

    // ! ❌不再需要「待步进的步数」，因为NARust-158是单线程的

    // ! ❌不复刻`finishedInputs`：仅DEBUG变量
    /// 最后一个输出之前的步数
    pub(in super::super) timer: usize,

    /// 静默等级（0~100）
    /// * 🚩【2024-06-27 19:06:32】不同于OpenNARS，此处仅使用普通整数
    pub(in super::super) silence_value: usize,

    /// 时间戳序列号（递增序列号）
    pub(in super::super) stamp_current_serial: ClockTime,

    /// shuffle用随机生成器
    /// * 🚩【2024-07-10 00:27:04】不应设置为全局变量：推理器之间不应共享数据
    /// * 🎯让推理结果可重复（而非随进程变化）
    pub(in super::super) shuffle_rng: StdRng,
}

/// 构造函数
impl Reasoner {
    /// 完全参数构造函数
    pub fn new(
        name: impl Into<String>,
        parameters: impl Into<Parameters>,
        inference_engine: impl Into<InferenceEngine>,
    ) -> Self {
        Self {
            name: name.into(),
            // * 🚩默认为空
            parameters: parameters.into(),
            memory: Memory::default(),
            recorder: ReasonRecorder::default(),
            io_channels: ReasonerChannels::default(),
            inference_engine: inference_engine.into(),
            derivation_datas: ReasonerDerivationData::default(),
            // * 🚩默认为0/false
            clock: 0,
            timer: 0,
            silence_value: 0,
            stamp_current_serial: 0,
            // * 🚩统一的随机数生成器
            shuffle_rng: Self::new_shuffle_rng(),
        }
    }

    fn new_shuffle_rng() -> StdRng {
        StdRng::seed_from_u64(0x137442)
    }
}

/// 功能性函数
impl Reasoner {
    /// 重置推理器
    pub fn reset(&mut self) {
        // * 🚩重置容器
        self.memory.init();
        self.derivation_datas.reset();
        self.recorder.reset();

        // * 🚩重置状态变量
        self.init_timer();
        self.clock = 0;
        self.stamp_current_serial = 0;

        // * 🚩重置全局变量
        crate::control::init_global_reason_parameters(); // 推理过程的全局参数（随机种子等）

        // * 🚩最后发送消息
        self.report_info("-----RESET-----");
    }

    /* 直接访问属性 */

    /// 获取推理器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取记忆区（不可变引用）
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// 获取记忆区（可变引用）
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// 获取超参数（不可变引用）
    pub fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    /// 获取静默等级
    pub fn silence_value(&self) -> usize {
        self.silence_value
    }

    /// 更新「当前时间戳序列号」
    /// * 📝OpenNARS中「先自增，再使用」
    pub fn updated_stamp_current_serial(&mut self) -> ClockTime {
        self.stamp_current_serial += 1;
        self.stamp_current_serial
    }

    /// 从内部「记录器」中拉取一个输出
    pub fn take_output(&mut self) -> Option<Output> {
        self.recorder.take()
    }
}
