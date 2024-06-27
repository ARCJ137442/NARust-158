//! 「转换推理上下文」
//!
//! ## Logs
//!
//! * ♻️【2024-06-27 12:54:19】开始根据改版OpenNARS重写

use std::ops::{Deref, DerefMut};

use navm::output::Output;

use super::{ReasonContext, ReasonContextCore, ReasonContextWithLinks};
use crate::{
    __delegate_from_core,
    control::{Parameters, Reasoner},
    entity::{Concept, RCTask, Task, TaskLink},
    global::{ClockTime, Float},
    storage::Memory,
};

/// 转换推理上下文
#[derive(Debug)]
pub struct ReasonContextTransform<'this> {
    /// 内部存储的「上下文核心」
    core: ReasonContextCore<'this>,

    /// 对「记忆区」的反向引用
    memory: &'this Memory,

    /// 选中的任务链
    /// * 📌【2024-05-21 20:26:30】不可空！
    /// * 📌构造后不重新赋值，但内部可变（预算推理/反馈预算值）
    current_task_link: TaskLink,
}

impl<'this> ReasonContextTransform<'this> {
    pub fn new<'r: 'this>(
        reasoner: &'r mut Reasoner,
        current_concept: Concept,
        current_task_link: TaskLink,
    ) -> Self {
        // * 🚩构造核心
        let core = ReasonContextCore::new(
            current_concept,
            &reasoner.parameters, // !【2024-06-26 23:55:17】此处需要直接使用字段，以证明借用不冲突
            reasoner.time(),
            reasoner.silence_value(),
        );
        Self {
            core,
            // * 🚩特有字段
            memory: &reasoner.memory,
            current_task_link,
        }
    }
}

impl ReasonContext for ReasonContextTransform<'_> {
    __delegate_from_core! {}

    fn memory(&self) -> &Memory {
        self.memory
    }

    fn current_task<'r, 's: 'r>(&'s self) -> impl Deref<Target = RCTask> + 'r {
        self.current_task_link.target_rc()
    }

    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = RCTask> + 'r {
        self.current_task_link.target_rc_mut()
    }

    fn absorbed_by_reasoner(mut self, reasoner: &mut Reasoner) {
        // * 🚩将「当前任务链」归还给「当前概念」（所有权转移）
        self.core // ! 📌必须分到不同字段
            .current_concept_mut()
            .put_task_link_back(self.current_task_link);
        // * 🚩从基类方法继续
        self.core.absorbed_by_reasoner(reasoner);
    }
}

impl ReasonContextWithLinks for ReasonContextTransform<'_> {
    fn current_belief(&self) -> Option<&crate::entity::JudgementV1> {
        // ! 📌「转换推理」的「当前信念」始终为空
        // * 🚩【2024-06-09 11:03:54】妥协：诸多「导出结论」需要使用「当前信念」，但所幸「当前信念」始终允许为空（方便作为默认值）
        None
    }

    fn belief_link_for_budget_inference(&mut self) -> Option<&mut crate::entity::TermLink> {
        // ! 📌「转换推理」的「当前信念链」始终为空
        // * 🚩【2024-06-09 11:03:54】妥协：诸多「预算推理」需要使用「当前信念链」，但「当前信念」在「概念推理」中不允许为空
        None
    }

    fn current_task_link(&self) -> &TaskLink {
        &self.current_task_link
    }

    fn current_task_link_mut(&mut self) -> &mut TaskLink {
        &mut self.current_task_link
    }
}
