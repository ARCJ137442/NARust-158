//! 「概念推理上下文」
//!
//! ## Logs
//!
//! * ♻️【2024-06-26 23:49:25】开始根据改版OpenNARS重写

use super::{ReasonContext, ReasonContextCore, ReasonContextCoreOut, ReasonContextWithLinks};
use crate::{
    __delegate_from_core,
    control::{Parameters, Reasoner},
    entity::{Concept, JudgementV1, RCTask, TLink, Task, TaskLink, TermLink},
    global::{ClockTime, Float},
    storage::Memory,
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;
use navm::output::Output;
use std::ops::{Deref, DerefMut};

/// 概念推理上下文
#[derive(Debug)]
pub struct ReasonContextConcept<'this> {
    /// 内部存储的「上下文核心」
    pub(crate) core: ReasonContextCore<'this>,
    /// 内部存储的「上下文输出」
    pub(crate) outs: ReasonContextCoreOut,

    /// 选中的任务链
    /// * 📌【2024-05-21 20:26:30】不可空！
    /// * 📌构造后不重新赋值，但内部可变（预算推理/反馈预算值）
    current_task_link: TaskLink,

    /// 选中的信念
    /// * 🚩【2024-05-30 09:25:15】内部不被修改，同时「语句」允许被随意复制（内容固定，占用小）
    current_belief: Option<JudgementV1>,

    /// 被选中的[词项链](TermLink)or信念链
    /// * 📝相比「转换推理上下文」仅多了个可查的「当前信念链」
    current_belief_link: TermLink,

    /// 🆕所有要参与「概念推理」的词项链（信念链）
    /// * 🎯装载「准备好的词项链（信念链）」，简化「概念推理准备阶段」的传参
    /// * 🚩目前对于「第一个要准备的词项链」会直接存储在「当前词项链（信念链）」中
    /// * 📌类似Rust所有权规则：始终只有一处持有「完全独占引用（所有权）」
    belief_links_to_reason: Vec<TermLink>,
}

impl<'this> ReasonContextConcept<'this> {
    /// 构造函数
    pub fn new<'r: 'this>(
        reasoner: &'r mut Reasoner,
        current_concept: Concept,
        current_task_link: TaskLink,
        mut belief_links_to_reason: Vec<TermLink>,
    ) -> Self {
        // * 🚩构造核心结构
        let core = ReasonContextCore::new(reasoner, current_concept);
        let outs = ReasonContextCoreOut::new();

        // * 🚩先将首个元素作为「当前信念链」
        debug_assert!(!belief_links_to_reason.is_empty());
        belief_links_to_reason.reverse(); // ! 将「待推理链接」反向，后续均使用pop方法
        let current_belief_link = belief_links_to_reason.pop().expect("待推理链接不应为空");

        // * 🚩构造自身
        let mut this = Self {
            core,
            outs,
            current_task_link,
            current_belief: None,
            current_belief_link,
            belief_links_to_reason,
        };

        // * 🚩从「当前信念链」出发，尝试获取并更新「当前信念」「新时间戳」
        // * 📝Rust中需要在构造后才调用方法
        this.update_current_belief();

        // 返回
        this
    }
}

impl ReasonContextConcept<'_> {
    /// 获取「当前信念链」
    pub fn current_belief_link(&self) -> &TermLink {
        &self.current_belief_link
    }

    /// 获取「当前信念链」（可变引用）
    /// ? 【2024-06-26 00:45:39】后续可做：内化「预算更新」，使之变为不可变引用
    pub fn current_belief_link_mut(&mut self) -> &mut TermLink {
        &mut self.current_belief_link
    }

    /// 切换到新的信念（与信念链）
    /// * 🚩返回值语义：`(是否切换成功, 在概念「词项链袋」中弹出的旧词项链)`
    /// * 📌【2024-05-21 10:26:59】现在是「概念推理上下文」独有
    /// * 🚩【2024-05-21 22:51:09】只在自身内部搬迁所有权：从「待推理词项链表」中取出一个「词项链」替代原有词项链
    /// * 🚩能取出⇒返回旧词项链，已空⇒返回`null`
    pub fn next_belief(&mut self) -> (bool, Option<TermLink>) {
        // * 🚩先尝试拿出下一个词项链，若拿不出则返回空值
        let mut current_belief_link = unwrap_or_return! {
            ?self.belief_links_to_reason.pop()
            // * 🚩 若没有更多词项链了⇒返回空表示「已结束」
            => (false, None)
        };

        // * 🚩交换拿到旧的值，更新「当前信念链」 | 此举保证「信念链」永不为空
        std::mem::swap(&mut self.current_belief_link, &mut current_belief_link);
        let old_term_link = current_belief_link;

        // * 🚩从「当前信念链」出发，尝试获取并更新「当前信念」「新时间戳」
        self.update_current_belief();

        // * ♻️回收弹出的旧词项链（所有权转移）
        let overflowed_old_link = self.current_concept_mut().put_term_link_back(old_term_link);

        // * 🚩收尾：返回被替换下来的「旧词项链」
        (true, overflowed_old_link)
    }

    fn update_current_belief(&mut self) {
        // * 🚩设置当前信念（可空性相对独立）
        self.current_belief = self.updated_current_belief();
    }

    /// 通过设置好的（非空的）「当前信念链」返回更新的「当前信念」（所有权）
    fn updated_current_belief(&self) -> Option<JudgementV1> {
        // * 🚩背景变量
        let new_belief_link = &self.current_belief_link;

        // * 🚩尝试从「当前信念链的目标」获取「当前信念」所对应的概念
        let belief_term = &*new_belief_link.target();
        let belief_concept = self.term_to_concept(belief_term)?;

        // * 🚩找到新的「信念」充当「当前信念」并返回（可空性相对独立）
        belief_concept
            .get_belief(&*self.current_task().get_())
            // * 🚩语句在此复制，以避开生命周期问题
            .cloned()
    }
}

impl ReasonContext for ReasonContextConcept<'_> {
    __delegate_from_core! {}

    fn current_task<'r, 's: 'r>(&'s self) -> impl Deref<Target = RCTask> + 'r {
        self.current_task_link.target_rc()
    }

    fn current_task_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = RCTask> + 'r {
        self.current_task_link.target_rc_mut()
    }

    fn absorbed_by_reasoner(mut self) {
        // * 🚩将最后一个「当前信念链」归还给「当前信念」（所有权转移）
        // * ❌此处只能销毁，不能报告：部分借用⇒借用冲突
        let _ = self
            .core
            .current_concept_mut()
            .put_term_link_back(self.current_belief_link);

        // * 🚩将「当前任务链」归还给「当前概念」（所有权转移）
        // * ❌此处只能销毁，不能报告：部分借用⇒借用冲突
        let _ = self
            .core
            .current_concept_mut()
            .put_task_link_back(self.current_task_link);

        // * 🚩销毁「当前信念」 | 变量值仅临时推理用
        drop(self.current_belief);

        // * 🚩吸收核心
        self.core.absorbed_by_reasoner(self.outs);
    }
}

impl ReasonContextWithLinks for ReasonContextConcept<'_> {
    fn current_belief(&self) -> Option<&JudgementV1> {
        self.current_belief.as_ref()
    }

    fn belief_link_for_budget_inference(&self) -> Option<&TermLink> {
        Some(&self.current_belief_link)
    }

    fn belief_link_for_budget_inference_mut(&mut self) -> Option<&mut TermLink> {
        Some(&mut self.current_belief_link)
    }

    fn current_task_link(&self) -> &TaskLink {
        &self.current_task_link
    }

    fn current_task_link_mut(&mut self) -> &mut TaskLink {
        &mut self.current_task_link
    }
}
