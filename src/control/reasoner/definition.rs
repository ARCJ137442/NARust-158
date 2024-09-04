//! 推理器 定义
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:02:36】开始根据改版OpenNARS重写

use super::ReasonRecorder;
use crate::{
    global::ClockTime,
    inference::InferenceEngine,
    parameters::Parameters,
    storage::{Memory, TaskBuffer},
    util::Serial,
};
use navm::output::Output;
use rand::{rngs::StdRng, SeedableRng};
use std::fmt::Debug;

// ! ❌【2024-06-27 18:01:23】不复刻静态常量`Reasoner.DEBUG`

/// 作为结构体的「推理器」
///
/// ⚠️【2024-08-11 15:52:04】函数指针、闭包等对象的 序列化/反序列化 问题
/// * 💭或将弃用有关「通道」「随机数生成器」等字段的序列反序列化可能，仅专注于「推理器存储」部分
///   * 亦即【可被序列化】的部分
/// * 🚩【2024-08-12 00:01:57】暂时搁置有关「全推理器」的序列化/反序列化
#[derive(Debug /* Serialize, Deserialize */)]
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

    /// 使用的推理引擎
    pub(in super::super) inference_engine: InferenceEngine,

    /// 推理过程的「中间数据」
    pub(in super::super) task_buffer: TaskBuffer,

    /// 系统时钟
    clock: ClockTime,

    // ! ❌不再状态「运行中」，因为NARust-158是始终运行的

    // ! ❌不再需要「待步进的步数」，因为NARust-158是单线程的

    // ! ❌不复刻`finishedInputs`：仅DEBUG变量
    // ! ❌不复刻`timer`「最后一个输出之前的步数」：这个量也是多线程OpenNARS才用的
    /// 音量等级（0~100）
    /// * 🚩【2024-06-27 19:06:32】不同于OpenNARS，此处仅使用普通整数
    volume: usize,

    /// 时间戳序列号（递增序列号）
    stamp_current_serial: ClockTime,

    /// 任务序列号（递增序列号）
    task_current_serial: Serial,

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
            inference_engine: inference_engine.into(),
            task_buffer: TaskBuffer::default(),
            // * 🚩默认为0/false
            clock: 0,
            volume: 0,
            stamp_current_serial: 0,
            task_current_serial: 0,
            // * 🚩统一的随机数生成器
            shuffle_rng: Self::new_shuffle_rng(),
        }
    }

    fn new_shuffle_rng() -> StdRng {
        StdRng::seed_from_u64(0x137442)
    }
}

/// 功能性函数
/// * ℹ️源自OpenNARS `class Reasoner`
/// * 📄核心字段的存取操作
///   * 🎯原则：尽量不暴露内部字段
impl Reasoner {
    /// 重置推理器
    pub fn reset(&mut self) {
        // * 🚩重置容器
        self.memory.init();
        self.task_buffer.reset();
        self.recorder.reset();

        // * 🚩重置状态变量
        self.clock = 0;
        self.stamp_current_serial = 0;
        self.task_current_serial = 0;

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

    /// 获取音量等级
    pub fn volume(&self) -> usize {
        self.volume
    }

    /// 🆕设置推理器「音量」
    /// * 📄参考内部字段[`Reasoner::volume`]
    pub fn set_volume(&mut self, volume: usize) {
        self.volume = volume;
    }

    /// 获取「当前时间戳序列号」
    /// * 🎯隔离内部字段实现
    pub fn stamp_current_serial(&self) -> ClockTime {
        self.stamp_current_serial
    }

    /// 设置当前时间戳序列号
    /// * 🎯序列反序列化中「覆盖当前时间戳序列号」
    /// * 🚩【2024-08-14 22:43:59】目前不对外公开
    pub(crate) fn set_stamp_current_serial(&mut self, value: ClockTime) {
        self.stamp_current_serial = value;
    }

    /// 更新「当前时间戳序列号」
    /// * 📝OpenNARS中「先自增，再使用」
    pub fn updated_stamp_current_serial(&mut self) -> ClockTime {
        self.stamp_current_serial += 1;
        self.stamp_current_serial
    }

    /// 获取「当前任务序列号」
    /// * 🎯隔离内部字段实现
    /// * ⚠️【2024-08-18 01:14:18】仅供内部「序列反序列化」使用
    pub(crate) fn task_current_serial(&self) -> Serial {
        self.task_current_serial
    }

    /// 设置当前任务序列号
    /// * 🎯序列反序列化中「覆盖当前任务序列号」
    /// * ⚠️【2024-08-18 01:14:18】仅供内部「序列反序列化」使用
    pub(crate) fn set_task_current_serial(&mut self, value: Serial) {
        self.task_current_serial = value;
    }

    /// 更新「当前任务序列号」
    /// * 📝OpenNARS中「先自增，再使用」
    /// * ⚠️【2024-08-18 01:14:18】仅供内部「序列反序列化」使用
    pub(crate) fn updated_task_current_serial(&mut self) -> Serial {
        self.task_current_serial += 1;
        self.task_current_serial
    }

    /// 获取时钟时间
    #[doc(alias = "clock")]
    pub fn time(&self) -> ClockTime {
        self.clock
    }

    /// 单步递进时钟时间
    pub fn tick(&mut self) {
        self.clock += 1;
    }

    /// 设置时钟时间
    /// * 🎯序列反序列化中「覆盖当前时间」
    /// * 🚩【2024-08-14 22:43:59】目前不对外公开
    #[doc(alias = "set_clock")]
    pub(crate) fn set_time(&mut self, value: ClockTime) {
        self.clock = value;
    }

    /// 从内部「记录器」中拉取一个输出
    pub fn take_output(&mut self) -> Option<Output> {
        self.recorder.take()
    }
}
