//! 基于「推理器」「推导上下文」有关「推理周期」的操作
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现OpenNARS
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌处理新任务(内部) from 工作周期
//!   * 📌处理新近任务(内部) from 工作周期
//!   * 📌处理概念(内部) from 工作周期
//!   * 📌立即处理(内部) from 处理新任务/处理新近任务
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️

use super::*;
use crate::{entity::*, inference::*, nars::*, storage::*, *};
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
        self.recorder_mut().put(Output::COMMENT {
            content: format!("--- Cycle {time} ---"),
        });

        // * 🚩🆕构建「直接推理上下文」
        let context: Self::DerivationContextDirect = DerivationContextDirect::new();

        // 本地直接推理 //
        let mut direct_process_result = self.__direct_process(context);

        // 概念高级推理 //
        if let ContextReady(ref mut context, ref mut term_links_to_process) = direct_process_result
        {
            // * 🚩正式开始「概念推理」
            self.__process_concept(context, term_links_to_process);
        }

        // 最终吸收上下文 //
        match direct_process_result {
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

    /// 🆕本地直接推理
    /// * 🚩最终只和「本地规则」[`LocalRules`]有关
    fn __direct_process(
        &mut self,
        mut context: Self::DerivationContextDirect,
    ) -> DirectProcessResult<
        Self::DerivationContextDirect,
        Self::DerivationContextReason,
        C::TermLink,
    > {
        // * 🚩处理新任务
        self.__process_new_task(&mut context);

        // TODO: `necessary?`可能也是自己需要考虑的问题：是否只在「处理无果」时继续
        if context.no_result() {
            // * 🚩处理新近任务
            self.__process_novel_task(&mut context);
        }

        // * 🚩过渡阶段 | 此处保留OpenNARS的做法，不把`no_result`判断放到「预处理」中
        match context.no_result() {
            true => self.__preprocess_concept_reason(context),
            false => DirectProcessResult::OnlyDirect(context),
        }
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
    ///   * 📄参见[`ReasonerWorkCycle::__process_concept`]
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
impl<C: ReasonContext, T: Reasoner<C>> ReasonerWorkCycle<C> for T {}
