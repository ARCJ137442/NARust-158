//! 直接推理上下文

use super::{ReasonContext, ReasonContextCore};
use crate::{
    __delegate_from_core,
    control::{Parameters, Reasoner},
    entity::{Concept, RCTask, Task},
    global::{ClockTime, Float},
    storage::Memory,
};
use navm::output::Output;

/// 🆕新的「直接推理上下文」对象
/// * 📄从「推理上下文」中派生，用于「概念-任务」的「直接推理」
#[derive(Debug)]
pub struct ReasonContextDirect<'this> {
    /// 内部存储的「上下文核心」
    core: ReasonContextCore<'this>,

    /// 对「记忆区」的反向引用
    /// * 🚩【2024-05-18 17:00:12】目前需要访问其「输出」「概念」等功能
    ///   * 📌需要是可变引用
    memory: &'this mut Memory,

    /// 选中的「任务」
    /// * 📌需要共享引用：从推理器的「共享引用池」中来
    current_task: RCTask,
}

impl<'this> ReasonContextDirect<'this> {
    pub fn new<'r: 'this>(
        reasoner: &'r mut Reasoner,
        current_concept: Concept,
        current_task: RCTask,
    ) -> Self {
        let core = ReasonContextCore::new(
            current_concept,
            &reasoner.parameters, // !【2024-06-26 23:55:17】此处需要直接使用字段，以证明借用不冲突
            reasoner.time(),
            reasoner.silence_value(),
        );
        Self {
            core,
            memory: &mut reasoner.memory,
            current_task,
        }
    }

    /// 📝对「记忆区」的可变引用，只在「直接推理」中可变
    pub fn memory_mut(&mut self) -> &mut Memory {
        self.memory
    }
}

impl ReasonContext for ReasonContextDirect<'_> {
    __delegate_from_core! {}

    fn memory(&self) -> &Memory {
        self.memory
    }

    fn current_task<'r, 's: 'r>(&'s self) -> impl std::ops::Deref<Target = RCTask> + 'r {
        &self.current_task
    }

    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl std::ops::DerefMut<Target = RCTask> + 'r {
        &mut self.current_task
    }

    fn absorbed_by_reasoner(self, reasoner: &mut Reasoner) {
        // * 🚩销毁核心
        self.core.absorbed_by_reasoner(reasoner);
        // * ✅Rust已在此处自动销毁剩余字段
    }
}
