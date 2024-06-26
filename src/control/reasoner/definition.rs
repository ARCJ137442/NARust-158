//! 推理器 定义
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 12:02:36】开始根据改版OpenNARS重写

use navm::output::Output;

use crate::{entity::RCTask, global::ClockTime, storage::Memory};

/// 作为结构体的「推理器」
pub struct Reasoner {
    // todo: 待实现
}

impl Reasoner {
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
