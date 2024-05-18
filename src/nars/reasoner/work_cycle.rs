//! 基于「推理器」「推导上下文」有关「推理周期」的操作
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现OpenNARS
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌吸收推理上下文(新)
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️

use super::*;
use crate::{entity::*, inference::*, nars::*};
use nar_dev_utils::list;
use navm::output::Output;

// TODO: 是否考虑「推理器」并吞「记忆区」
// * 💡如：就将「记忆区」变成一个纯粹的「增强版概念袋」使用

/// 🆕推理器「概念推理」的结果
pub enum DirectProcessResult<D, R, T>
where
    D: ReasonContext,
    R: ReasonContext,
    T: TermLink,
{
    /// * 📌情况1：「直接推理」完毕，但因各种情况没打算开始「概念推理」
    ///   * 📄如：先前的「直接推理」已有结果，不太需要「概念推理」
    ///   * 📄如：未能选中「当前概念」「当前任务链」
    ///   * 📄如：选中的「任务链」类型是「转换」，只需应用NAL-4相关规则
    OnlyDirect(D),
    /// * 📌情况2：「直接推理」之后，已构建好「推理上下文」并可以开始「概念推理」
    ///   * ⚠️此时「当前概念」「当前任务链」「当前任务」均已准备好
    ///   * ✨亦包含【将要参与轮流`reason`】的「词项链列表」
    ContextReady(R, Vec<T>),
}

/// 推理器与「工作周期」有关的功能
pub trait ReasonerWorkCycle<C: ReasonContext>: Reasoner<C> {
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
    fn work_cycle(&mut self) {
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

        // 变量预置工作 //
        use DirectProcessResult::*;

        // ! 🚩【2024-05-17 16:24:25】↓现在迁移到推理器中，直接变为相应字段
        let time = self.clock();
        self.report(Output::COMMENT {
            content: format!("--- Cycle {time} ---"),
        });

        // * 🚩🆕构建「直接推理上下文」
        let mut context = DerivationContextDirect::new();

        // 本地直接推理 //
        // * 🚩此处保留OpenNARS的做法，不把`no_result`判断放到「预处理」中
        let need_concept_process = self.__direct_process(&mut context);

        // 过渡阶段 //
        // * 🎯准备选择概念、任务链、词项链
        // * ⚠️在其中也进行部分推理：NAL-4「结构转换」
        let mut result = match need_concept_process {
            true => self.__preprocess_concept_reason(context),
            false => OnlyDirect(context),
        };

        // 概念高级推理 //
        if let ContextReady(ref mut context, ref mut term_links_to_process) = result {
            // * 🚩正式开始「概念推理」
            self.__process_concept(context, term_links_to_process);
        }

        // 最终吸收上下文 //
        match result {
            OnlyDirect(context) => self.__absorb_reasoned_context(context),
            ContextReady(context, ..) => self.__absorb_reasoned_context(context),
        }

        // self.__novel_tasks().refresh(); // ! ❌【2024-05-08 14:49:48】这个方法是「观察者」用的，此处不用
    }

    /// 🆕吸收经过推理后积累的「推理上下文」
    /// * 🚩【2024-05-18 00:47:41】目前不区分「直接推理」与「概念推理」
    ///   * 📌只需其共有的「缓存之结果」不变即可
    fn __absorb_reasoned_context<Context>(&mut self, mut context: Context)
    where
        Context: DerivationContext<C>,
    {
        // * 🚩将其中新增加的「导出任务」加入自身之中
        let buffer = list![
            new_task
            while let Some(new_task) = (context.__new_tasks_mut().pop())
        ];
        for new_task in buffer.into_iter().rev() {
            // * 🚩倒转遍历输出
            self.__new_tasks_mut().push_back(new_task);
        }

        // * 🚩将其中新增加的「输出」放入缓存
        let buffer = list![
            new_task
            while let Some(new_task) = (context.new_outputs_mut().pop())
        ];
        for output in buffer.into_iter().rev() {
            // * 🚩倒转遍历输出
            self.report(output);
        }
    }
}

/// 通过「批量实现」自动加功能
impl<C: ReasonContext, T: Reasoner<C>> ReasonerWorkCycle<C> for T {}
