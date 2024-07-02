//! 直接推理上下文

use super::{ReasonContext, ReasonContextCore, ReasonContextCoreOut};
use crate::{
    __delegate_from_core,
    control::{Parameters, Reasoner},
    entity::{Concept, RCTask, Task},
    global::{ClockTime, Float},
    language::Term,
    storage::Memory,
};
use navm::output::Output;

/// 🆕新的「直接推理上下文」对象
/// * 📄从「推理上下文」中派生，用于「概念-任务」的「直接推理」
#[derive(Debug)]
pub struct ReasonContextDirect<'this> {
    /// 内部存储的「上下文核心」
    pub(crate) core: ReasonContextCore<'this>,
    /// 内部存储的「上下文输出」
    pub(crate) outs: ReasonContextCoreOut,

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
        let outs = ReasonContextCoreOut::new();
        Self {
            core,
            outs,
            current_task,
        }
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        self.core.memory_mut()
    }

    /// 获取「已存在的概念」（从「键」出发，可变引用）
    /// * 🎯在「概念链接到任务」中使用
    pub fn key_to_concept_mut(&mut self, key: &str) -> Option<&mut Concept> {
        match key == Memory::term_to_key(self.current_term()) {
            true => Some(self.current_concept_mut()),
            false => self.memory_mut().key_to_concept_mut(key),
        }
    }

    /// 获取「已存在的概念」或创建（从「键」出发，可变引用）
    /// * 🎯在「概念链接到任务」中使用（子概念→自身，或递归处理时）
    pub fn get_concept_or_create(&mut self, term: &Term) -> Option<&mut Concept> {
        match term == self.current_term() {
            true => Some(self.current_concept_mut()),
            false => self.memory_mut().get_concept_or_create(term),
        }
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
        self.core.absorbed_by_reasoner(self.outs);
        // * ✅Rust已在此处自动销毁剩余字段
    }
}
