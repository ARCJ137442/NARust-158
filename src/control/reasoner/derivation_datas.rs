//! 存放推理器的「推理数据」
//! * 🎯存储有关「新任务列表」「新近任务袋」的数据
//! * 📄新任务列表
//! * 📄新近任务袋
//! * ⚠️不缓存「NAVM输出」：输出保存在[「推理记录器」](super::report)中

use crate::{
    control::DEFAULT_PARAMETERS,
    entity::{RCTask, Task},
    storage::Bag,
    util::IterInnerRcSelf,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 🚀推理导出用数据
/// * 📌【2024-08-12 20:26:44】内部所存储的「任务」暂时无需考虑「任务共享引用归一化」问题
///   * ⚠️本来要考虑的「任务共享引用」：在每个「任务」内部的「父任务」
#[derive(Debug, Serialize, Deserialize)]
pub(in super::super) struct ReasonerDerivationData {
    /// 新任务列表
    /// * 🚩没有上限，不适合作为「缓冲区」使用
    ///
    /// # 📄OpenNARS
    ///
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    pub new_tasks: VecDeque<Task>,

    /// 新近任务袋
    /// * ⚠️因「作为【共享引用】的任务」不满足[`Item`]，故不使用[`RCTask`]
    pub novel_tasks: Bag<Task>,
}

impl Default for ReasonerDerivationData {
    fn default() -> Self {
        // 对「新近任务袋」当「概念袋」使
        // TODO: 🏗️后续仍有待解耦与「概念袋」的联系——分离「超参数」中的默认值
        let novel_tasks = Bag::new(
            DEFAULT_PARAMETERS.concept_bag_size,
            DEFAULT_PARAMETERS.concept_forgetting_cycle,
        );
        Self {
            new_tasks: Default::default(),
            novel_tasks,
        }
    }
}

impl ReasonerDerivationData {
    /// 重置推理导出数据
    /// * 🎯原先是「推理器」代码的一部分
    pub fn reset(&mut self) {
        self.new_tasks.clear();
        self.novel_tasks.init();
    }
}

/// 为「推理器导出数据」添加功能
/// * ⚠️【2024-06-27 23:12:13】此处不能为推理器添加
///   * 📄在[`crate::control::Reasoner::load_from_new_tasks`]中，需要明确借用以避免借用冲突（冲突with记忆区）
impl ReasonerDerivationData {
    /// 添加新任务
    /// * 🚩【2024-06-27 20:32:38】不使用[`RCTask`]，并且尽可能限制「共享引用」的使用
    pub fn add_new_task(&mut self, task: Task) {
        self.new_tasks.push_back(task);
    }

    // !  🚩【2024-06-28 00:15:43】废弃：实际使用中只需`if let pop`
    // /// 判断「是否有新任务」
    // pub fn has_new_task(&self) -> bool {
    //     !self.new_tasks.is_empty()
    // }

    /// 从「新任务」中拿出（第）一个任务
    #[doc(alias = "take_a_new_task")]
    #[must_use]
    pub fn pop_new_task(&mut self) -> Option<Task> {
        self.new_tasks.pop_front()
    }

    /// 将一个任务放进「新近任务袋」
    /// * 🚩同时返回「溢出的新近任务」
    #[must_use]
    pub fn put_in_novel_tasks(&mut self, task: Task) -> Option<Task> {
        self.novel_tasks.put_in(task)
    }

    /// 从「新近任务袋」拿出一个任务
    #[must_use]
    pub fn take_a_novel_task(&mut self) -> Option<Task> {
        self.novel_tasks.take_out()
    }
}

/// 用于「序列反序列化」的功能
impl ReasonerDerivationData {
    /// 遍历其中所有「共享任务引用」的可变引用
    /// * 🚩若直接存储
    pub(crate) fn iter_mut_task_rcs(&mut self) -> impl Iterator<Item = &mut RCTask> {
        self.new_tasks
            .iter_mut()
            .chain(self.novel_tasks.iter_mut())
            .flat_map(|t| t.iter_inner_rc_self())
    }

    #[cfg(test)]
    pub(crate) fn iter_task_rcs(&self) -> impl Iterator<Item = &RCTask> {
        self.new_tasks
            .iter()
            .chain(self.novel_tasks.iter())
            .flat_map(Task::parent_task)
    }
}
