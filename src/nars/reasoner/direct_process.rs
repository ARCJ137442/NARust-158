//! 推理器有关「直接推理/立即推理」的功能
//! * 🎯模拟以`Memory.immediateProcess`为入口的「直接推理」
//! * 🎯将其中有关「直接推理」的代码摘录出来
//!   * 📌处理新任务(内部) from 工作周期(@记忆区)
//!   * 📌处理新近任务(内部) from 工作周期(@记忆区)
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//!   * 📌直接处理 from 立即处理(@记忆区)
//!   * 📌处理判断(内部 @概念) from 直接处理
//!   * 📌处理问题(内部 @概念) from 直接处理
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组

use crate::{entity::*, inference::*, nars::*, storage::*, *};
use navm::output::Output;

/// 推理器与「工作周期」有关的功能
pub trait ReasonerDirectProcess<C: ReasonContext>: Reasoner<C> {
    /// 🆕本地直接推理
    /// * 🚩最终只和「本地规则」[`LocalRules`]有关
    fn __direct_process(&mut self, context: &mut Self::DerivationContextDirect) -> bool {
        // * 🚩处理新任务
        self.__process_new_task(context);

        // TODO: `necessary?`可能也是自己需要考虑的问题：是否只在「处理无果」时继续
        if context.no_result() {
            // * 🚩处理新近任务
            self.__process_novel_task(context);
        }

        // * 🚩返回「是否要继续」 | 不与「概念推理」的功能耦合
        !context.no_result()
    }

    /// 模拟`Memory.processNewTask`
    /// * 🚩【2024-05-17 21:25:46】从「记忆区」换成「直接推理上下文」
    ///
    /// # 📄OpenNARS
    ///
    /// Process the newTasks accumulated in the previous workCycle, accept input
    /// ones and those that corresponding to existing concepts, plus one from the
    /// buffer.
    fn __process_new_task(&mut self, context: &mut Self::DerivationContextDirect) {
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
        while let Some(task) = self.__new_tasks_mut().pop_front() {
            let task_concent = task.content();
            if task.is_input() || self.memory().term_to_concept(task_concent).is_some() {
                self.__immediate_process(task, context);
            } else {
                let sentence = task.sentence();
                if let SentenceType::Judgement(truth) = sentence.punctuation() {
                    let d = truth.expectation();
                    if d > DEFAULT_PARAMETERS.default_creation_expectation {
                        self.__novel_tasks_mut().put_in(task);
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
    fn __process_novel_task(&mut self, context: &mut Self::DerivationContextDirect) {
        /* 📄OpenNARS源码：
        Task task = novelTasks.takeOut(); // select a task from novelTasks
        if (task != null) {
            immediateProcess(task);
        } */
        let task = self.__novel_tasks_mut().take_out();
        if let Some(task) = task {
            self.__immediate_process(task, context);
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
    fn __immediate_process(&mut self, task: C::Task, context: &mut Self::DerivationContextDirect) {
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
        *context.current_task_mut() = Some(task);
        // * 🚩【2024-05-17 21:28:14】放入后又拿出，以在「置入所有权后获取其引用」
        let task = context.current_task().as_ref().unwrap();
        // ! 🚩【2024-05-08 16:07:06】此处不得不使用大量`clone`以解决借用问题；后续可能是性能瓶颈
        let current_term = task.content().clone();
        let budget = task.budget().clone();
        if let Some(current_concept) = self.memory_mut().get_concept_or_create(&current_term) {
            let key = current_concept.____key_cloned(); // ! 此处亦需复制，以免借用问题
            self.memory_mut().activate_concept(&key, &budget);
        }
    }
}

/// 通过「批量实现」自动加功能
impl<C: ReasonContext, T: Reasoner<C>> ReasonerDirectProcess<C> for T {}
