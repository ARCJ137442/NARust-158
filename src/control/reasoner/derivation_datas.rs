//! 存放推理器的「推理数据」
//! * 🎯存储有关「新任务列表」「新近任务袋」的数据
//! * 📄新任务列表
//! * 📄新近任务袋
//! * ⚠️不缓存「NAVM输出」：输出保存在[「推理记录器」](super::report)中

use nar_dev_utils::unwrap_or_return;

use super::Reasoner;
use crate::{entity::Task, storage::Bag, util::ToDisplayAndBrief};
use std::collections::VecDeque;

/// 🚀推理导出用数据
#[derive(Debug, Default)]
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

impl ReasonerDerivationData {
    /// 重置推理导出数据
    /// * 🎯原先是「推理器」代码的一部分
    pub fn reset(&mut self) {
        self.new_tasks.clear();
        self.novel_tasks.init();
    }
}

/// 为「推理器」添加功能
impl Reasoner {
    /// 添加新任务
    /// * 🚩【2024-06-27 20:32:38】不使用[`RCTask`]，并且尽可能限制「共享引用」的使用
    pub fn add_new_task(&mut self, task: Task) {
        self.derivation_datas.new_tasks.push_back(task);
    }

    /// 判断「是否有新任务」
    pub fn has_new_task(&self) -> bool {
        !self.derivation_datas.new_tasks.is_empty()
    }

    /// 从「新任务」中拿出（第）一个任务
    pub fn pop_new_task(&mut self) -> Option<Task> {
        self.derivation_datas.new_tasks.pop_front()
    }

    /// 将一个任务放进「新近任务袋」
    /// * 🚩同时销毁「溢出的新近任务」
    pub fn pub_in_novel_tasks(&mut self, task: Task) {
        let overflowed = unwrap_or_return!(?self.derivation_datas.novel_tasks.put_in(task) => ());
        // 🆕🚩报告「任务溢出」
        self.report(navm::output::Output::COMMENT {
            content: format!(
                "!!! NovelTasks overflowed: {}",
                overflowed.to_display_long()
            ),
        })
    }

    /// 从「新近任务袋」拿出一个任务
    pub fn take_a_novel_task(&mut self) -> Option<Task> {
        self.derivation_datas.novel_tasks.take_out()
    }
}
