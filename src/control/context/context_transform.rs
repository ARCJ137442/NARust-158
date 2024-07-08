//! 「转换推理上下文」
//!
//! ## Logs
//!
//! * ♻️【2024-06-27 12:54:19】开始根据改版OpenNARS重写

use super::{ReasonContext, ReasonContextCore, ReasonContextCoreOut, ReasonContextWithLinks};
use crate::{
    __delegate_from_core,
    control::{Parameters, Reasoner},
    entity::{Concept, RCTask, Task, TaskLink},
    global::{ClockTime, Float},
    storage::Memory,
};
use navm::output::Output;
use std::ops::{Deref, DerefMut};

/// 转换推理上下文
#[derive(Debug)]
pub struct ReasonContextTransform<'this> {
    /// 内部存储的「上下文核心」
    pub(crate) core: ReasonContextCore<'this>,
    /// 内部存储的「上下文输出」
    pub(crate) outs: ReasonContextCoreOut,

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
        let core = ReasonContextCore::new(reasoner, current_concept);
        let outs = ReasonContextCoreOut::new();
        Self {
            core,
            outs,
            // * 🚩特有字段
            current_task_link,
        }
    }
}

impl ReasonContext for ReasonContextTransform<'_> {
    __delegate_from_core! {}

    fn current_task<'r, 's: 'r>(&'s self) -> impl Deref<Target = RCTask> + 'r {
        self.current_task_link.target_rc()
    }

    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = RCTask> + 'r {
        self.current_task_link.target_rc_mut()
    }

    fn absorbed_by_reasoner(mut self) {
        // * 🚩将「当前任务链」归还给「当前概念」（所有权转移）
        // * 📝此处只能销毁：会有「部分借用」的问题
        let _ = self
            .core // ! 📌必须分到不同字段
            .current_concept_mut()
            .put_task_link_back(self.current_task_link);
        // * 🚩从基类方法继续
        self.core.absorbed_by_reasoner(self.outs);
    }
}

impl ReasonContextWithLinks for ReasonContextTransform<'_> {
    fn current_belief(&self) -> Option<&crate::entity::JudgementV1> {
        // ! 📌「转换推理」的「当前信念」始终为空
        // * 🚩【2024-06-09 11:03:54】妥协：诸多「导出结论」需要使用「当前信念」，但所幸「当前信念」始终允许为空（方便作为默认值）
        None
    }

    fn belief_link_for_budget_inference(&self) -> Option<&crate::entity::TermLink> {
        // ! 📌「转换推理」的「当前信念链」始终为空
        // * 🚩【2024-06-09 11:03:54】妥协：诸多「预算推理」需要使用「当前信念链」，但「当前信念」在「概念推理」中不允许为空
        None
    }

    fn belief_link_for_budget_inference_mut(&mut self) -> Option<&mut crate::entity::TermLink> {
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
