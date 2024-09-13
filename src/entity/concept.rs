//! 🎯复刻OpenNARS `nars.entity.Concept`
//!
//! * ♻️【2024-06-24 18:59:59】开始基于改版OpenNARS重写

use crate::{
    entity::{
        BudgetValue, Item, Judgement, JudgementV1, RCTask, Sentence, TaskLink, TermLink,
        TermLinkTemplate, Token,
    },
    global::{ClockTime, Float},
    inference::{Budget, BudgetFunctions},
    language::Term,
    parameters::{Parameters, DEFAULT_PARAMETERS},
    storage::{ArrayBuffer, ArrayRankTable, Bag, Buffer, IsCompatibleToAddF, RankF, RankTable},
    util::{to_display_when_has_content, Iterable, ToDisplayAndBrief},
};
use nar_dev_utils::{join, RefCount};
use serde::{Deserialize, Serialize};

/// 复刻改版OpenNARS `nars.entity.Concept`
///
/// # 📄OpenNARS
///
/// A concept contains information associated with a term, including directly and indirectly related tasks and beliefs.
/// <p>
/// To make sure the space will be released, the only allowed reference to a  concept are those in a ConceptBag.
///
/// All other access go through the Term that names the concept.
#[derive(Debug, Serialize, Deserialize)]
pub struct Concept {
    /// 🆕Item令牌
    token: Token,

    /// The term is the unique ID of the concept
    term: Term,

    /// Task links for indirect processing
    task_links: Bag<TaskLink>,

    /// Term links between the term and its components and compounds
    term_links: Bag<TermLink>,

    /// Link templates of TermLink, only in concepts with CompoundTerm
    /// * 🎯用于「复合词项构建词项链」如「链接到任务」
    /// * 📌【2024-06-04 20:14:09】目前确定为「所有『内部元素』链接到自身的可能情况」的模板集
    /// * 📝只会创建「从内部元素链接到自身」（target=）
    /// * 📝在[`ConceptLinking::prepareTermLinkTemplates`]中被准备，随后不再变化
    link_templates_to_self: Vec<TermLinkTemplate>,

    /// Question directly asked about the term
    /// * 📝需要是共享引用：一个「问题」既然是一个「任务」，那除了被存储在这缓冲区内，还会被「任务链」引用
    /// * 🚩【2024-07-02 15:58:38】转换为共享引用
    questions: ArrayBuffer<RCTask>,

    /// 信念表
    ///
    /// * 📝【2024-08-11 23:23:42】对接[`serde`]序列反序列化 经验笔记
    ///   * 📌一个应对「带函数指针结构」的serde模式：白板结构+指针覆写
    ///     * ❓关键问题：这里「要覆写的指针」从哪儿决定
    ///   * 💡一个核心可利用信息：反序列化时可以「基于字段指定要专门反序列化该字段的函数」
    ///     * ✨因此：正巧「信念表」的函数指针是由「信念表」这个字段决定的
    ///   * 🚩最终做法：通过「特制的反序列化函数」实现函数指针的无损序列反序列化
    ///   * ! 💫踩坑：基于「中间类型」的方式较为繁琐
    ///     * ⚠️需要包装旧有类型：对原先代码侵入式大
    ///     * ℹ️实际上需要借助「中间类型」，多出许多boilerplate
    ///
    /// # 📄OpenNARS
    ///
    ///  Sentences directly made about the term, with non-future tense
    #[serde(deserialize_with = "beliefs::deserialize")]
    beliefs: ArrayRankTable<JudgementV1>,
}

/// 有关「信念排行表」的模块
mod beliefs {
    use super::*;
    pub const RANK_F: RankF<JudgementV1> = BudgetValue::rank_belief;
    pub const IS_COMPATIBLE_TO_ADD_F: IsCompatibleToAddF<JudgementV1> = belief_compatible_to_add;

    type Table = ArrayRankTable<JudgementV1>;

    /// 构造一个「信念排行表」
    pub fn new(capacity: usize) -> Table {
        Table::new(
            capacity,
            RANK_F, // * 📌作为「预算函数」的「预算值」
            IS_COMPATIBLE_TO_ADD_F,
        )
    }

    /// 信念适合添加的条件：不能等价
    fn belief_compatible_to_add(incoming: &impl Judgement, existed: &impl Judgement) -> bool {
        // * 📌【2024-07-09 17:13:29】debug：应该是「不等价⇒可兼容」
        !incoming.is_belief_equivalent(existed)
    }

    /// 定制版序列化函数
    /// * 🚩反序列化→覆写指针→原样返回
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ArrayRankTable<JudgementV1>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 先反序列化出原排行表
        let mut table = ArrayRankTable::deserialize(deserializer)?;
        // 再覆盖函数指针
        table.override_fn(beliefs::RANK_F, beliefs::IS_COMPATIBLE_TO_ADD_F);
        // 最后返回
        Ok(table)
    }
}

/// 用于构造「概念」的结构体
/// * 🎯构造函数中规范传参
/// * ⚠️保留后续被修改的可能
#[derive(Debug, Clone, Copy)]
pub struct ConceptParameters {
    task_link_forgetting_cycle: usize,
    term_link_forgetting_cycle: usize,
    maximum_questions_length: usize,
    maximum_belief_length: usize,
    task_link_bag_size: usize,
    term_link_bag_size: usize,
}

impl From<&Parameters> for ConceptParameters {
    fn from(parameters: &Parameters) -> Self {
        Self {
            task_link_forgetting_cycle: parameters.task_link_forgetting_cycle,
            term_link_forgetting_cycle: parameters.term_link_forgetting_cycle,
            maximum_questions_length: parameters.maximum_questions_length,
            maximum_belief_length: parameters.maximum_belief_length,
            task_link_bag_size: parameters.task_link_bag_size,
            term_link_bag_size: parameters.term_link_bag_size,
        }
    }
}

impl Concept {
    /// 🆕完全参数构造函数
    /// * 🚩包括两个「超参数」的引入
    /// * 📝OpenNARS改版中不引入任何有关「记忆区」「概念链接」这些控制机制中的元素
    /// * 🚩【2024-08-16 16:01:01】目前还是直接引入「超参数」类型为好
    ///   * 💭省去大量传参担忧
    pub fn new(
        term: Term,
        parameters: ConceptParameters,
        initial_budget: BudgetValue,
        link_templates_to_self: Vec<TermLinkTemplate>,
    ) -> Self {
        // 解构参数
        let ConceptParameters {
            maximum_questions_length,
            maximum_belief_length,
            task_link_bag_size,
            term_link_bag_size,
            task_link_forgetting_cycle,
            term_link_forgetting_cycle,
        } = parameters;
        // 创建内部字段
        let token = Token::new(term.name(), initial_budget);
        let questions = ArrayBuffer::new(maximum_questions_length);
        let beliefs = beliefs::new(maximum_belief_length);
        let task_links = Bag::new(task_link_forgetting_cycle, task_link_bag_size);
        let term_links = Bag::new(term_link_forgetting_cycle, term_link_bag_size);
        // 创建结构体
        Self {
            token,
            term,
            task_links,
            term_links,
            link_templates_to_self,
            questions,
            beliefs,
        }
    }

    /// 🆕对外接口：获取「当前信念表」
    /// * 🎯从「直接推理」而来
    /// * 🚩【2024-07-02 16:23:51】目前因「无需获取内部表」，直接返回迭代器
    pub fn beliefs(&self) -> impl Iterator<Item = &JudgementV1> {
        self.beliefs.iter()
    }

    /// * 🚩添加到固定容量的缓冲区，并返回溢出的那个（溢出==所添加 ⇒ 添加失败）
    ///
    /// # 📄OpenNARS
    ///
    /// Add a new belief (or goal) into the table Sort the beliefs/goals by rank,
    /// and remove redundant or low rank one
    #[must_use]
    pub fn add_belief(&mut self, belief: JudgementV1) -> Option<JudgementV1> {
        self.beliefs.add(belief)
    }

    /// 🆕对外接口：获取「当前所有问题」
    /// * 🎯从「直接推理」而来
    /// * 📝有可能是「拿着问题找答案」：此时引用无需可变
    /// * 🚩【2024-07-02 16:23:51】目前因「无需获取内部表」，直接返回迭代器
    pub fn questions(&self) -> impl Iterator<Item = &RCTask> {
        self.questions.iter()
    }

    /// 🆕对外接口：获取「当前所有问题」
    /// * 🎯从「直接推理」而来
    /// * ⚠️需要可变引用：要在过程中「设置最优解」
    /// * 🚩【2024-07-02 16:23:51】目前因「无需获取内部表」，直接返回迭代器
    pub fn questions_mut(&mut self) -> impl Iterator<Item = &mut RCTask> {
        self.questions.iter_mut()
    }

    /// 🆕对外接口：添加问题到「问题集」
    /// * 🚩除了「添加」以外，还会实行「任务缓冲区」机制
    #[must_use]
    pub fn add_question(&mut self, question: RCTask) -> Option<RCTask> {
        self.questions.add(question)
    }

    /// API方法 @ 链接建立
    ///
    /// # 📄OpenNARS
    ///
    /// Return the templates for TermLinks,
    /// only called in Memory.continuedProcess
    pub fn link_templates_to_self(&self) -> &[TermLinkTemplate] {
        &self.link_templates_to_self
    }

    /// 🆕API方法 @ 链接建立
    pub fn put_in_term_link(&mut self, link: TermLink) -> Option<TermLink> {
        self.term_links.put_in(link)
    }

    /// 🆕API方法 @ 链接建立
    #[must_use]
    pub fn put_in_task_link(&mut self, link: TaskLink) -> Option<TaskLink> {
        self.task_links.put_in(link)
    }

    /// 🆕从「任务链袋」获取一个任务链
    /// * 🚩仅用于「概念推理」
    #[must_use]
    pub fn take_out_task_link(&mut self) -> Option<TaskLink> {
        self.task_links.take_out()
    }

    /// 🆕将一个任务链放回「任务链袋」
    /// * 🚩仅用于「概念推理」
    #[must_use]
    pub fn put_task_link_back(&mut self, link: TaskLink) -> Option<TaskLink> {
        self.task_links.put_back(link)
    }

    /// 🆕将一个词项链放回「词项链袋」
    /// * 🚩仅用于「概念推理」
    #[must_use]
    pub fn put_term_link_back(&mut self, link: TermLink) -> Option<TermLink> {
        self.term_links.put_back(link)
    }

    /// # 📄OpenNARS
    ///
    /// Return the associated term, called from Memory only
    pub fn term(&self) -> &Term {
        &self.term
    }

    /// # 📄OpenNARS
    ///
    /// Recalculate the quality of the concept [to be refined to show extension/intension balance]
    pub fn term_links_average_priority(&self) -> Float {
        self.term_links.average_priority()
    }

    /// # 📄OpenNARS
    /// Select a isBelief to interact with the given task in inference get the first qualified one
    ///
    /// only called in RuleTables.reason
    /// * 📝⚠️实际上并不`only called in RuleTables.reason`
    /// * 📄在「组合规则」的「回答带变量合取」时用到
    /// * 🚩改：去除其中「设置当前时间戳」的副作用，将其迁移到调用者处
    pub fn get_belief(&self, task_sentence: &impl Sentence) -> Option<&JudgementV1> {
        // * 🚩此处按「信念排名」从大到小遍历；第一个满足「证据基不重复」的信念将被抽取
        for belief in self.beliefs.iter() {
            // * 📝在OpenNARS 3.0.4中会被覆盖：
            // * 📄`nal.setTheNewStamp(taskStamp, belief.stamp, currentTime);`
            // * ✅【2024-06-08 10:13:46】现在彻底删除newStamp字段，不再需要覆盖了
            if !task_sentence.evidential_overlap(belief) {
                let selected = belief;
                return Some(selected);
            }
        }
        None
    }

    /// # 📄OpenNARS改版
    ///
    /// * 📌特殊的「根据任务链拿出词项链（信念链）」
    /// Replace default to prevent repeated inference, by checking TaskLink
    /// * 🔗ProcessReason.chooseTermLinksToReason
    /// * 🎯在「概念推理」的「准备待推理词项链」的过程中用到
    pub fn take_out_term_link_from_task_link(
        &mut self,
        task_link: &mut TaskLink,
        time: ClockTime,
    ) -> Option<TermLink> {
        for _ in 0..DEFAULT_PARAMETERS.max_matched_term_link {
            // * 🚩尝试拿出词项链 | 📝此间存在资源竞争
            // * ✅此处已包括「没有词项链⇒返回空值」的逻辑
            let term_link = self.term_links.take_out()?;
            // * 🚩任务链相对词项链「新近」⇒直接返回
            if task_link.novel(&term_link, time) {
                return Some(term_link);
            }
            // * 🚩当即放回（可能会销毁旧的词项链）
            let _ = self.term_links.put_back(term_link);
        }
        None
    }

    /// 🆕迭代内部所有可能的「任务」
    /// * ⚠️不保证内容不重复
    /// * 🎯呈现推理器内所有现存的「任务」
    /// * 📄目前参考的点儿
    ///   * 任务链袋
    ///   * 问题缓冲区
    pub(crate) fn iter_tasks(&self) -> impl Iterator<Item = &RCTask> {
        let iter_task_links = self.iter_task_links().map(TaskLink::target_rc_ref);
        let iter_questions = self.iter_questions();
        iter_task_links.chain(iter_questions)
    }

    /// 🆕迭代内部所有的信念
    pub(crate) fn iter_beliefs(&self) -> impl Iterator<Item = &JudgementV1> {
        self.beliefs.iter()
    }

    /// 🆕迭代内部所有的问题（任务）
    pub(crate) fn iter_questions(&self) -> impl Iterator<Item = &RCTask> {
        self.questions.iter()
    }

    /// 🆕迭代内部所有的词项链
    pub(crate) fn iter_term_links(&self) -> impl Iterator<Item = &TermLink> {
        self.term_links.iter()
    }

    /// 🆕迭代内部所有的任务链
    pub(crate) fn iter_task_links(&self) -> impl Iterator<Item = &TaskLink> {
        self.task_links.iter()
    }

    /// 🆕迭代内部所有的「任务共享引用」
    /// * 🎯序列反序列化中「归一任务共享引用」的需要
    /// * 🚩取材自「任务链」「问题表」
    pub(crate) fn iter_tasks_mut(&mut self) -> impl Iterator<Item = &mut RCTask> {
        let iter_task_links = self.task_links.iter_mut().map(TaskLink::target_rc_ref_mut);
        let iter_questions = self.questions.iter_mut();
        iter_task_links.chain(iter_questions)
    }
}

impl Budget for Concept {
    fn priority(&self) -> super::ShortFloat {
        self.token.priority()
    }

    fn __priority_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__priority_mut()
    }

    fn durability(&self) -> super::ShortFloat {
        self.token.durability()
    }

    fn __durability_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__durability_mut()
    }

    fn quality(&self) -> super::ShortFloat {
        self.token.quality()
    }

    fn __quality_mut(&mut self) -> &mut super::ShortFloat {
        self.token.__quality_mut()
    }
}

impl Item for Concept {
    fn key(&self) -> &String {
        self.token.key()
    }
}

/// 🆕是否在[`Concept::to_display`]处显示更细致的内容
/// * 🎯与主类解耦
const DETAILED_STRING: bool = false;

impl ToDisplayAndBrief for Concept {
    fn to_display(&self) -> String {
        match DETAILED_STRING {
            true => join! {
                => self.token.budget().to_display_brief()
                => " "
                => self.key()
            },
            false => self.key().into(),
        }
    }

    fn to_display_long(&self) -> String {
        let mut base = join! {
            => self.to_display_brief()
            => to_display_when_has_content("  term_links: ", self.term_links.to_display())
            => to_display_when_has_content("  task_links: ", self.task_links.to_display())
        };
        if !self.questions.is_empty() {
            base += "\n  questions:";
            for t in self.questions.iter() {
                base += "\n";
                base += &t.get_().to_display();
            }
        }
        if !self.beliefs.is_empty() {
            base += "\n  beliefs:";
            for b in self.beliefs.iter() {
                base += "\n";
                base += &b.to_display();
            }
        }
        base
    }
}
