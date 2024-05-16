//! 基于「推导上下文」对「记忆区」有关「推理周期」的操作
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌处理新任务(内部) from 工作周期
//!   * 📌处理新近任务(内部) from 工作周期
//!   * 📌处理概念(内部) from 工作周期
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//! * ❗包含主控到具体推理的直接入口
//!   TODO: 后续或考虑基于「推理器」而非「推导上下文」
//!
//! * ✅【2024-05-12 16:10:24】基本迁移完所有功能

use crate::{entity::*, inference::*, nars::DEFAULT_PARAMETERS, storage::*, *};
use navm::output::Output;

/// 记忆区处理：整理与「记忆区」有关的操作
/// * 🚩目前以「记忆区」为中心，以便从「记忆区」处添加方法
/// * 🚩【2024-05-12 15:00:59】因为`RuleTables::transform_task(self);`，要求[`Sized`]
pub trait MemoryCycleProcess: DerivationContext {
    /* ---------- system working workCycle ---------- */

    /// 模拟`Memory.workCycle`
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic working cycle of the system: process new Tasks, then fire a concept
    ///
    /// Called from Reasoner.tick only
    ///
    /// @param clock The current time to be displayed
    fn work_cycle(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        recorder.append(" --- " + clock + " ---\n");
        processNewTask();
        if (noResult()) { // necessary?
            processNovelTask();
        }
        if (noResult()) { // necessary?
            processConcept();
        }
        novelTasks.refresh(); */
        let time = self.time(); // ! 🚩【2024-05-08 15:38:00】必须先获取：借用问题
        self.report(Output::COMMENT {
            content: format!("--- Cycle {time} ---"),
        });
        self.__process_new_task(memory);
        // TODO: `necessary?`可能也是自己需要考虑的问题：是否只在「处理无果」时继续
        if self.no_result() {
            // * 🚩🆕【2024-05-08 14:49:27】合并条件
            self.__process_novel_task(memory);
            self.__process_concept(memory);
        }
        // self.__novel_tasks().refresh(); // ! ❌【2024-05-08 14:49:48】这个方法是「观察者」用的，此处不用
    }

    /// 模拟`Memory.processNewTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Process the newTasks accumulated in the previous workCycle, accept input
    /// ones and those that corresponding to existing concepts, plus one from the
    /// buffer.
    fn __process_new_task(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        Task task;
        int counter = newTasks.size(); // don't include new tasks produced in the current workCycle
        while (counter-- > 0) {
            task = newTasks.removeFirst();
            if (task.isInput() || (termToConcept(task.getContent()) != null)) { // new input or existing concept
                immediateProcess(task);
            } else {
                Sentence s = task.getSentence();
                if (s.isJudgment()) {
                    double d = s.getTruth().getExpectation();
                    if (d > Parameters.DEFAULT_CREATION_EXPECTATION) {
                        novelTasks.putIn(task); // new concept formation
                    } else {
                        recorder.append("!!! Neglected: " + task + "\n");
                    }
                }
            }
        } */
        // let mut task;
        // // * 🚩逆序遍历，实际上又是做了个`-->`语法
        // for counter in (0..self.__new_tasks().len()).rev() {
        //     task = self.__new_tasks_mut().pop_front();
        // }
        // ! ❌【2024-05-08 14:55:26】莫只是照抄OpenNARS的逻辑：此处只是要「倒序取出」而已
        while let Some(task) = memory.__new_tasks_mut().pop_front() {
            let task_concent = task.content();
            if task.is_input() || memory.term_to_concept(task_concent).is_some() {
                self.__immediate_process(task, memory);
            } else {
                let sentence = task.sentence();
                if let SentenceType::Judgement(truth) = sentence.punctuation() {
                    let d = truth.expectation();
                    if d > DEFAULT_PARAMETERS.default_creation_expectation {
                        memory.__novel_tasks_mut().put_in(task);
                    } else {
                        self.report(Output::COMMENT {
                            content: format!("!!! Neglected: {}", task.to_display_long()),
                        });
                    }
                }
            }
        }
    }

    /// 模拟`Memory.processNovelTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Select a novel task to process.
    fn __process_novel_task(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        Task task = novelTasks.takeOut(); // select a task from novelTasks
        if (task != null) {
            immediateProcess(task);
        } */
        let task = memory.__novel_tasks_mut().take_out();
        if let Some(task) = task {
            self.__immediate_process(task, memory);
        }
    }

    /// 模拟`Memory.processConcept`
    ///
    /// # 📄OpenNARS
    ///
    /// Select a concept to fire.
    fn __process_concept(&mut self, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        currentConcept = concepts.takeOut();
        if (currentConcept != null) {
            currentTerm = currentConcept.getTerm();
            recorder.append(" * Selected Concept: " + currentTerm + "\n");
            concepts.putBack(currentConcept); // current Concept remains in the bag all the time
            currentConcept.fire(); // a working workCycle
        } */
        let concept = memory.__concepts_mut().take_out();
        if let Some(current_concept) = concept {
            let current_term = current_concept.term();
            self.report(Output::COMMENT {
                // * 🚩【2024-05-07 23:05:14】目前仍是将词项转换为「词法Narsese」
                content: format!("* Selected Concept: {}", current_term),
            });
            let key = current_concept.key().clone(); // * 🚩🆕【2024-05-08 15:08:22】拷贝「元素id」以便在「放回」之后仍然能索引
            memory.__concepts_mut().put_back(current_concept);
            // current_concept.fire(); // ! ❌【2024-05-08 15:09:04】不采用：放回了还用，将导致引用混乱
            let concept = memory.__concepts_mut().get_mut(&key).expect("不可能失败");
            // * 💡后续或许也把「当前概念」放到「推导上下文」中，仅在最后「回收上下文」时开始
            self.__fire_concept(concept);
        }
    }

    /* ---------- task processing ---------- */

    /// 模拟`Memory.immediateProcess`
    /// * 📝OpenNARS中对「任务处理」都需要在「常数时间」中运行完毕
    ///   * 💡【2024-05-08 15:34:49】这也是为何「可交换词项变量匹配」需要伪随机「shuffle」
    ///
    /// # 📄OpenNARS
    ///
    /// Immediate processing of a new task,
    /// in constant time Local processing,
    /// in one concept only
    ///
    /// @param task the task to be accepted
    fn __immediate_process(&mut self, task: Self::Task, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
        currentTask = task; // one of the two places where concept variable is set
        recorder.append("!!! Insert: " + task + "\n");
        currentTerm = task.getContent();
        currentConcept = getConcept(currentTerm);
        if (currentConcept != null) {
            activateConcept(currentConcept, task.getBudget());
            currentConcept.directProcess(task);
        } */
        self.report(Output::COMMENT {
            content: format!("!!! Insert: {}", task.to_display_long()),
        });
        *self.current_task_mut() = task;
        // ! 🚩【2024-05-08 16:07:06】此处不得不使用大量`clone`以解决借用问题；后续可能是性能瓶颈
        let task = /* &** */self.current_task();
        let current_term = task.content().clone();
        let budget = task.budget().clone();
        if let Some(current_concept) = memory.get_concept_or_create(&current_term) {
            let key = current_concept.____key_cloned(); // ! 此处亦需复制，以免借用问题
            memory.activate_concept(&key, &budget);
        }
    }
}

/// 自动实现，以便添加方法
impl<T: DerivationContext> MemoryCycleProcess for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
