//! 推理器有关「概念推理/高级推理」的功能
//! * 🎯模拟以`RuleTables.reason`为入口的「概念推理」
//!   * 📌处理概念(内部) from 工作周期
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-05-18 16:36:06】目前从「推理周期」迁移出来

use super::*;
use crate::{entity::*, inference::*, nars::*, storage::*, *};
use navm::output::Output;

/// 推理器与「概念推理」有关的功能
pub trait ReasonerConceptProcess<C: ReasonContext>: Reasoner<C> {
    /// 🆕在「直接推理」与「概念推理」之间的「过渡部分」
    /// * 🚩选择概念、选择任务链、选择预备词项链
    /// * 🚩【2024-05-18 00:49:01】需要传入整个上下文所有权，以便在其中构建「推理上下文」
    fn __preprocess_concept_reason(
        &mut self,
        mut context: Self::DerivationContextDirect,
    ) -> DirectProcessResult<
        Self::DerivationContextDirect,
        Self::DerivationContextReason,
        C::TermLink,
    > {
        use DirectProcessResult::*;

        // * 🚩选中概念
        let concept = self.memory_mut().__concepts_mut().take_out();
        if let Some(current_concept) = concept {
            let current_term = current_concept.term();
            self.report(Output::COMMENT {
                content: format!("* Selected Concept: {current_term}"),
            });
            let key = current_concept.key().clone(); // * 🚩🆕【2024-05-08 15:08:22】拷贝「元素id」以便在「放回」之后仍然能索引
            self.memory_mut().__concepts_mut().put_back(current_concept);
            let current_concept = self.memory_mut().__concepts_mut().get_mut(&key).unwrap();

            // * 🚩选中任务
            let current_task_link = current_concept.__task_links_mut().take_out();
            if let Some(current_task_link) = current_task_link {
                // ! 🚩【2024-05-08 16:19:31】必须在「修改」之前先报告（读取）
                self.report(Output::COMMENT {
                    content: format!(
                        "* Selected TaskLink: {}",
                        current_task_link.target().to_display_long()
                    ),
                });

                // * 🚩开始载入完整的「上下文」内容
                *context.current_task_link_mut() = Some(current_task_link);
                // ? 【2024-05-08 15:41:21】↓这个有意义吗
                *context.current_belief_link_mut() = None;
                // ! 🚩【2024-05-08 16:21:32】目前为「引用计数」需要，暂时如此引入（后续需要解决…）
                // * 💫【2024-05-17 22:00:24】本来是「当前任务就是『当前任务链对应的任务』」，但却被「组合规则/decomposeStatement」硬生生打破
                //   * ❗导致徒增诸多「引用计数」等麻烦
                // TODO: 【2024-05-17 22:01:52】后续一定要处理好这个关系，至少先把那个例外给解决掉
                let current_task_link = context.current_task_link().as_ref().unwrap();
                let task = current_task_link.target();
                *context.current_task_mut() = Some(task.clone());

                // * 🚩处理「任务链」中特殊的「转换」情况：使用NAL-4单独处理「任务转换」规则
                let direct_process_result = if let TermLinkRef::Transform(..) =
                    context.current_task_link().as_ref().unwrap().type_ref()
                {
                    *context.current_belief_mut() = None;
                    // let current_task_link = self.current_task_link();
                    context.transform_task();
                    // * 🚩【2024-05-17 22:05:21】没进入真正的`reason`，没有上下文结果
                    DirectProcessResult::OnlyDirect(context)
                }
                // * 🚩过了所有特殊情况，开始准备「概念推理」
                else {
                    // * 尝试构建
                    let build_result =
                        context.build(self.memory(), self.clock(), self.silence_value());
                    match build_result {
                        Ok(context_reason) => {
                            // * 💭【2024-05-18 01:31:54】按OpenNARS原意，不论是否有，总归是能产生`Vec`的

                            // * 🚩准备任务链 | 【2024-05-08 16:52:41】先收集，再处理——避免重复借用
                            let current_concept =
                                self.memory_mut().__concepts_mut().get_mut(&key).unwrap();
                            let term_links_to_process =
                                Self::__choose_term_links_to_reason(current_concept);

                            // * 🚩最终返回，可用做推理
                            ContextReady(context_reason, term_links_to_process)
                        }
                        Err(mut context_direct) => {
                            // * 💭【2024-05-18 01:27:19】这是个异常情况，应该报告
                            context_direct.report(Output::ERROR {
                                // TODO: 是否后续要更详细些，「要求`Debug`」传染的问题
                                description: format!("!!! Failed to build context: {}", ""),
                            });
                            OnlyDirect(context_direct)
                        }
                    }
                };
                return direct_process_result;
            }
        }
        // * 🚩没选出完整的上下文内容：直接返回，后续不再进行「概念推理」
        OnlyDirect(context)
    }

    /// 🆕围绕任务链，获取可推理的词项链列表
    #[inline]
    fn __choose_term_links_to_reason(current_concept: &mut C::Concept) -> Vec<C::TermLink> {
        let mut term_links = vec![];
        // * 🆕🚩【2024-05-08 16:55:53】简化：实际上只是「最多尝试指定次数下，到了就不尝试」
        for _ in 0..DEFAULT_PARAMETERS.max_reasoned_term_link {
            let term_link = current_concept.__term_links_mut().take_out();
            match term_link {
                Some(term_link) => term_links.push(term_link),
                None => break,
            }
        }
        term_links
    }

    /// 模拟`Memory.processConcept`
    /// * 🚩【2024-05-18 00:39:53】现在一定传入一个「概念推导上下文」，没有回旋余地
    ///   * 🔬根据在OpenNARS的魔改实验，得：「直接推理」的结果可以只是「继续(预备好的上下文) / 终止」的枚举
    ///   * 📌故只需在「结果为『继续』」时将「预备好的上下文」传入
    /// * ⚠️此处的实际功能已经与OpenNARS有所出入
    ///   * 📌实际上这里已经选好了「概念」与「要推理的词项链」
    ///
    /// # 📄OpenNARS
    ///
    /// Select a concept to fire.
    fn __process_concept(
        &mut self,
        context: &mut Self::DerivationContextReason,
        term_links_to_process: &mut Vec<C::TermLink>,
    ) {
        /* 📄OpenNARS源码：
        currentConcept = concepts.takeOut();
        if (currentConcept != null) {
            currentTerm = currentConcept.getTerm();
            recorder.append(" * Selected Concept: " + currentTerm + "\n");
            concepts.putBack(currentConcept); // current Concept remains in the bag all the time
            currentConcept.fire(); // a working workCycle
        } */
        // TODO: 此处只需「让任务链轮流与词项链擦出火花」即可
        // * 先点一次火
        Self::__fire_concept(context);
        while let Some(mut term_link) = term_links_to_process.pop() {
            self.report(Output::COMMENT {
                content: format!(
                    "* Selected TermLink: {}",
                    term_link.target().to_display_long()
                ),
            });
            // * 🚩改变信念链（词项链）以便复用上下文 | 交换并取出「已推理完的词项链」
            std::mem::swap(context.current_belief_link_mut(), &mut term_link);
            // * 「点火」
            Self::__fire_concept(context);
            // * 放回「已推理完的词项链」
            context
                .current_concept_mut()
                .__term_links_mut()
                .put_back(term_link);
        }
        // TODO: 【2024-05-18 11:53:53】但此时「最后一个词项链」仍然在「推理上下文」里边
        // TODO: 【2024-05-18 11:53:58】需要在「吸收推理上下文」中特别处理——💡通过一个特别的特征方法「解构」拆分所有权
    }

    /* ---------- main loop ---------- */

    /// 🆕模拟`Concept.fire`
    /// * 📌【2024-05-08 15:06:09】不能让「概念」干「记忆区」干的事
    /// * 📝OpenNARS中从「记忆区」的[「处理概念」](Memory::process_concept)方法中调用
    /// * ⚠️依赖：[`crate::inference::RuleTables`]
    /// * 🚩【2024-05-12 16:08:58】现在独立在「推导上下文」中，
    ///   * 📌只是会被不断更改「当前词项链」以便「多次使用`reason`」
    ///   * 📌最后会返回上下文，以备最终吸收
    /// * 🚩【2024-05-18 00:51:54】现在传参不能直接用「概念」的引用，否则会有「重复引用」问题
    /// * ⚠️此处代码现已不与OpenNARS相同
    ///   * 📄参见[`ReasonerConceptProcess::__process_concept`]
    ///
    /// # 📄OpenNARS
    ///
    /// An atomic step in a concept, only called in {@link Memory#processConcept}
    fn __fire_concept(context: &mut Self::DerivationContextReason)
    /* -> Self::DerivationContextReason */
    {
        /* 📄OpenNARS源码：
        TaskLink currentTaskLink = taskLinks.takeOut();
        if (currentTaskLink == null) {
            return;
        }
        memory.currentTaskLink = currentTaskLink;
        memory.currentBeliefLink = null;
        memory.getRecorder().append(" * Selected TaskLink: " + currentTaskLink + "\n");
        Task task = currentTaskLink.getTargetTask();
        memory.currentTask = task; // one of the two places where concept variable is set
        // memory.getRecorder().append(" * Selected Task: " + task + "\n"); // for
        // debugging
        if (currentTaskLink.getType() == TermLink.TRANSFORM) {
            memory.currentBelief = null;
            RuleTables.transformTask(currentTaskLink, memory); // to turn concept into structural inference as below?
        } else {
            int termLinkCount = Parameters.MAX_REASONED_TERM_LINK;
            // while (memory.noResult() && (termLinkCount > 0)) {
            while (termLinkCount > 0) {
                TermLink termLink = termLinks.takeOut(currentTaskLink, memory.getTime());
                if (termLink != null) {
                    memory.getRecorder().append(" * Selected TermLink: " + termLink + "\n");
                    memory.currentBeliefLink = termLink;
                    RuleTables.reason(currentTaskLink, termLink, memory);
                    termLinks.putBack(termLink);
                    termLinkCount--;
                } else {
                    termLinkCount = 0;
                }
            }
        }
        taskLinks.putBack(currentTaskLink); */

        // * 🔥启动推理
        RuleTables::reason(context);
    }
}

/// 通过「批量实现」自动加功能
impl<C: ReasonContext, T: Reasoner<C>> ReasonerConceptProcess<C> for T {}
