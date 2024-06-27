//! 🎯复刻OpenNARS `nars.entity.Concept`
//!
//! * ♻️【2024-06-24 18:59:59】开始基于改版OpenNARS重写

use crate::{
    control::{Parameters, DEFAULT_PARAMETERS},
    entity::{
        BudgetValue, Item, Judgement, JudgementV1, Sentence, Task, TaskLink, TermLink,
        TermLinkTemplate, Token,
    },
    global::{ClockTime, Float},
    inference::{Budget, BudgetFunctions},
    language::Term,
    storage::{ArrayBuffer, ArrayRankTable, Bag, Buffer, RankTable},
    util::{Iterable, ToDisplayAndBrief},
};
use nar_dev_utils::join;
use std::usize;

/// 复刻改版OpenNARS `nars.entity.Concept`
///
/// # 📄OpenNARS
///
/// A concept contains information associated with a term, including directly and indirectly related tasks and beliefs.
/// <p>
/// To make sure the space will be released, the only allowed reference to a  concept are those in a ConceptBag.
///
/// All other access go through the Term that names the concept.
#[derive(Debug)]
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
    questions: ArrayBuffer<Task>,

    /// Sentences directly made about the term, with non-future tense
    beliefs: ArrayRankTable<JudgementV1>,
}

impl Concept {
    /// 🆕完全参数构造函数
    /// * 🚩包括两个「超参数」的引入
    /// * 📝OpenNARS改版中不引入任何有关「记忆区」「概念链接」这些控制机制中的元素
    pub fn new(
        term: Term,
        task_link_forgetting_rate: usize,
        term_link_forgetting_rate: usize,
        initial_budget: BudgetValue,
        link_templates_to_self: Vec<TermLinkTemplate>,
    ) -> Self {
        const PARAMETERS: Parameters = DEFAULT_PARAMETERS;
        let token = Token::new(term.name(), initial_budget);
        let questions = ArrayBuffer::new(PARAMETERS.maximum_questions_length);
        let beliefs = ArrayRankTable::new(
            PARAMETERS.maximum_belief_length,
            BudgetValue::rank_belief, // * 📌作为「预算函数」的「预算值」
            Self::belief_compatible_to_add,
        );
        let task_links = Bag::new(task_link_forgetting_rate, PARAMETERS.task_link_bag_size);
        let term_links = Bag::new(term_link_forgetting_rate, PARAMETERS.term_link_bag_size);
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

    fn belief_compatible_to_add(incoming: &impl Judgement, existed: &impl Judgement) -> bool {
        incoming.is_belief_equivalent(existed)
    }

    /// 🆕对外接口：获取「当前信念表」
    /// * 🎯从「直接推理」而来
    pub fn beliefs(&self) -> &ArrayRankTable<JudgementV1> {
        &self.beliefs
    }

    /// * 🚩添加到固定容量的缓冲区，并返回溢出的那个（溢出==所添加 ⇒ 添加失败）
    ///
    /// # 📄OpenNARS
    ///
    /// Add a new belief (or goal) into the table Sort the beliefs/goals by rank,
    /// and remove redundant or low rank one
    pub fn add_belief(&mut self, belief: JudgementV1) -> Option<JudgementV1> {
        self.beliefs.add(belief)
    }

    /// 🆕对外接口：获取「当前所有问题」
    /// * 🎯从「直接推理」而来
    pub fn questions(&self) -> &ArrayBuffer<Task> {
        &self.questions
    }

    /// 🆕对外接口：添加问题到「问题集」
    /// * 🚩除了「添加」以外，还会实行「任务缓冲区」机制
    pub fn add_question(&mut self, question: Task) -> Option<Task> {
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
    pub fn put_in_task_link(&mut self, link: TaskLink) -> Option<TaskLink> {
        self.task_links.put_in(link)
    }

    /// 🆕从「任务链袋」获取一个任务链
    /// * 🚩仅用于「概念推理」
    pub fn take_out_task_link(&mut self) -> Option<TaskLink> {
        self.task_links.take_out()
    }

    /// 🆕将一个任务链放回「任务链袋」
    /// * 🚩仅用于「概念推理」
    pub fn put_task_link_back(&mut self, link: TaskLink) -> Option<TaskLink> {
        self.task_links.put_back(link)
    }

    /// 🆕将一个词项链放回「词项链袋」
    /// * 🚩仅用于「概念推理」
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
            => " "
            => self.key()
            => "\nterm_links: " => self.term_links.to_display()
            => "\ntask_links: " => self.task_links.to_display()
        };
        base += "\nquestions:";
        for t in self.questions.iter() {
            base += "\n";
            base += &t.to_display();
        }
        base += "\nbeliefs:";
        for b in self.beliefs.iter() {
            base += "\n";
            base += &b.to_display();
        }
        base
    }
}
