//! NARS推理器中有关「任务推理循环」的功能
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现

use super::*;

// TODO: 具体功能迁移 from _memory_process
// TODO: 是否考虑「推理器」并吞「记忆区」
// * 💡如：就将「记忆区」变成一个纯粹的「增强版概念袋」使用

pub trait ReasonerWorkCycle: Reasoner {
    // TODO: 推理循环
    fn work_cycle(&mut self) {}
}

/// 通过「批量实现」自动加功能
impl<T: Reasoner> ReasonerWorkCycle for T {}
