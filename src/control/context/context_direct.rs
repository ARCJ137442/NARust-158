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
    pub(crate) core: ReasonContextCore<'this>,

    /// 选中的「任务」
    /// * 📌需要共享引用：从推理器的「共享引用池」中来
    pub(crate) current_task: RCTask,
}

impl<'this> ReasonContextDirect<'this> {
    pub fn new<'r: 'this>(
        reasoner: &'r mut Reasoner,
        current_concept: Concept,
        current_task: RCTask,
    ) -> Self {
        let core = ReasonContextCore::new(reasoner, current_concept);
        Self { core, current_task }
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        self.core.memory_mut()
    }
}

impl ReasonContext for ReasonContextDirect<'_> {
    __delegate_from_core! {}

    fn current_task<'r, 's: 'r>(&'s self) -> impl std::ops::Deref<Target = RCTask> + 'r {
        &self.current_task
    }

    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl std::ops::DerefMut<Target = RCTask> + 'r {
        &mut self.current_task
    }

    fn absorbed_by_reasoner(self) {
        // * 🚩销毁核心
        self.core.absorbed_by_reasoner();
        // * ✅Rust已在此处自动销毁剩余字段
    }
}
