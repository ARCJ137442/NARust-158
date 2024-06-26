//! 直接推理上下文

use super::{ReasonContext, ReasonContextCore};
use crate::{
    control::{Parameters, Reasoner},
    entity::{Concept, RCTask, Task},
    global::{ClockTime, Float},
    storage::Memory,
};
use navm::output::Output;

/// 🆕新的「直接推理上下文」对象
/// * 📄从「推理上下文」中派生，用于「概念-任务」的「直接推理」
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
    fn memory(&self) -> &Memory {
        self.memory
    }

    fn time(&self) -> ClockTime {
        self.core.time()
    }

    fn parameters(&self) -> &Parameters {
        self.core.parameters()
    }

    fn silence_percent(&self) -> Float {
        self.core.silence_percent()
    }

    fn num_new_tasks(&self) -> usize {
        self.core.num_new_tasks()
    }

    fn add_new_task(&mut self, task: Task) {
        self.core.add_new_task(task)
    }

    fn add_output(&mut self, output: Output) {
        self.core.add_output(output)
    }

    fn current_concept(&self) -> &Concept {
        self.core.current_concept()
    }

    fn current_concept_mut(&mut self) -> &mut Concept {
        self.core.current_concept_mut()
    }

    fn current_task(&self) -> &RCTask {
        &self.current_task
    }

    fn current_task_mut(&mut self) -> &mut RCTask {
        &mut self.current_task
    }

    fn absorbed_by_reasoner(self, reasoner: &mut Reasoner) {
        // * 🚩销毁核心
        self.core.absorbed_by_reasoner(reasoner);
        // * ✅Rust已在此处自动销毁剩余字段
    }
}
