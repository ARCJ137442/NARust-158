//! 「概念处理」主模块
//! * 🎯有关「概念推理」的主控
//!   * 📌信念获取 from 组合规则、规则表
//!   * 📌添加入表 from 处理判断
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部) from 直接处理
//!   * 📌处理问题(内部) from 直接处理
//!   * 📌「点火」 from 处理概念(@记忆区)
//!
//! * ♻️【2024-05-16 18:07:08】初步独立成模块功能

use crate::{control::*, entity::*, global::ClockTime, types::TypeContext};

/// 有关「概念」的处理
/// * 🎯分离NARS控制机制中有关「概念」的部分
/// * 📌此处均有关「直接推理」
///   * 📝OpenNARS中均由`Memory.immediateProcess`调用
pub trait ConceptProcessDirect<C: TypeContext>: DerivationContextDirect<C> {
    /* ---------- direct processing of tasks ---------- */

    /// 模拟`Concept.getBelief`
    /// * 📝OpenNARS用在「组合规则」与「推理上下文构建」中
    ///   * ✅「组合规则」中就是正常使用「推理上下文」：其「概念」就是「推理上下文」中使用到的「当前概念」
    ///   * ⚠️「推理上下文构建」中要同时获取「&mut 推理上下文」与「&概念」
    ///     * 🚩【2024-05-17 15:07:02】因此全部解耦：直接传引用
    /// * 🚩【2024-05-16 18:43:40】因为是「赋值『新时间戳』到上下文」，故需要`self`可变
    ///
    /// # 📄OpenNARS
    ///
    /// Select a isBelief to interact with the given task in inference
    ///
    /// get the first qualified one
    ///
    /// only called in RuleTables.reason
    ///
    /// @param task The selected task
    /// @return The selected isBelief
    fn get_belief(
        new_stamp_mut: &mut Option<C::Stamp>,
        time: ClockTime,
        concept: &C::Concept,
        task: &C::Task,
    ) -> Option<C::Sentence> {
        /* 📄OpenNARS源码：
        Sentence taskSentence = task.getSentence();
        for (Sentence belief : beliefs) {
            memory.getRecorder().append(" * Selected Belief: " + belief + "\n");
            memory.newStamp = Stamp.make(taskSentence.getStamp(), belief.getStamp(), memory.getTime());
            if (memory.newStamp != null) {
                Sentence belief2 = (Sentence) belief.clone(); // will this mess up priority adjustment?
                return belief2;
            }
        }
        return null; */
        let task_sentence = task.sentence();
        for belief in concept.__beliefs() {
            let new_stamp = C::Stamp::from_merge(task_sentence.stamp(), belief.stamp(), time);
            if new_stamp.is_some() {
                let belief2 = belief.clone();
                return Some(belief2);
            }
            // * 🚩必须赋值，无论是否有
            *new_stamp_mut = new_stamp;
        }
        None
    }

    /// 模拟`Concept.addToTable`
    /// * 🚩实际上是个静态方法：不依赖实例
    /// * 🚩对「物品列表」使用标准库的[`Vec`]类型，与[`Concept::__beliefs_mut`]同步
    ///
    /// # 📄OpenNARS
    ///
    /// Add a new belief (or goal) into the table Sort the beliefs/goals by rank,
    /// and remove redundant or low rank one
    ///
    /// @param newSentence The judgment to be processed
    /// @param table       The table to be revised
    /// @param capacity    The capacity of the table
    fn __add_to_table(sentence: &C::Sentence, table: &mut Vec<C::Sentence>, capacity: usize) {
        /* 📄OpenNARS源码：
        float rank1 = BudgetFunctions.rankBelief(newSentence); // for the new isBelief
        Sentence judgment2;
        float rank2;
        int i;
        for (i = 0; i < table.size(); i++) {
            judgment2 = table.get(i);
            rank2 = BudgetFunctions.rankBelief(judgment2);
            if (rank1 >= rank2) {
                if (newSentence.equivalentTo(judgment2)) {
                    return;
                }
                table.add(i, newSentence);
                break;
            }
        }
        if (table.size() >= capacity) {
            while (table.size() > capacity) {
                table.remove(table.size() - 1);
            }
        } else if (i == table.size()) {
            table.add(newSentence);
        } */
        todo!("// TODO: 有待实现")
    }
}

/// 自动实现，以便添加方法
impl<C: TypeContext, T: DerivationContextDirect<C>> ConceptProcessDirect<C> for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
