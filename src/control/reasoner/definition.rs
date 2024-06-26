//! 推理器 定义
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:02:36】开始根据改版OpenNARS重写

use crate::{control::Parameters, entity::RCTask, global::ClockTime, storage::Memory};
use navm::output::Output;

/// 作为结构体的「推理器」
pub struct Reasoner {
    /// * 📌【2024-06-26 23:55:40】需要部分公开，以便在其它地方解决「借用冲突」问题
    pub(in crate::control) parameters: Parameters,
    pub(in crate::control) memory: Memory,
    // TODO
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
