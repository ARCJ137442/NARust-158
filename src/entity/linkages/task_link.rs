//! 🎯复刻OpenNARS `nars.entity.TaskLink`
//! * ♻️【2024-06-22 12:02:13】开始基于OpenNARS改版重写

use super::{TLink, TLinkType, TLinkage, TermLink, TermLinkTemplate};
use crate::{
    control::DEFAULT_PARAMETERS,
    entity::{BudgetValue, Item, RCTask, Sentence, ShortFloat, Task, Token},
    global::ClockTime,
    inference::{Budget, Evidential},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::join;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Reference to a Task.
///
/// The reason to separate a Task and a TaskLink is that the same Task can be linked from multiple Concepts, with different BudgetValue.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskLink {
    /// 内部链接到的任务（共享引用）
    inner: TLinkage<RCTask>,

    /// 🆕Item令牌
    token: Token,

    /// * 📌记忆【曾经匹配过的词项链】的索引键和时间（序列号）
    /// * 🎯用于推理中判断[「是否新近」](TaskLink::novel)
    /// * 🚩【2024-06-22 12:31:20】仍然可用定长数组存储
    ///   * ℹ️虽然定长，但可能包含未初始化空间
    ///   * 📌对这些「未初始化空间」采用「默认值填充」的方式
    /// * 📌【2024-06-22 12:53:25】完全可以使用元组合二为一、统一长度
    ///   * 🚩【2024-06-22 12:53:41】目前采用该方式
    ///   * 📍结构：`(索引键, 时间)`
    ///
    /// # 📄OpenNARS
    ///
    /// - Remember the TermLinks that has been used recently with this TaskLink
    /// - Remember the time when each TermLink is used with this TaskLink
    recorded_links: Box<[(String, ClockTime)]>,

    /// The number of TermLinks remembered
    /// * 📌记忆【曾经匹配过的词项链】的个数
    /// * 🎯用于推理中判断[「是否新近」](TaskLink::novel)
    n_recorded_term_links: usize,
}

impl TaskLink {
    /// 直接获取内部链接到的「任务引用」
    /// * 🎯用于上级「概念」收集所有「任务引用」
    pub(in crate::entity) fn target_rc_ref(&self) -> &RCTask {
        &self.inner.target
    }
    /// 直接获取内部链接到的「任务引用」（可变）
    /// * 🎯用于「序列反序列化」「归一化任务共享引用」
    /// * ⚠️慎用
    pub(in crate::entity) fn target_rc_ref_mut(&mut self) -> &mut RCTask {
        &mut self.inner.target
    }

    pub fn target_rc<'r, 's: 'r>(&'s self) -> impl Deref<Target = RCTask> + 'r {
        // ! 🚩【2024-06-22 12:21:12】要直接引用target字段，不能套两层`impl Deref`
        // * * ️📝会导致「临时变量引用」问题
        &self.inner.target
    }

    pub fn target_rc_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = RCTask> + 'r {
        // ! 🚩【2024-06-22 12:21:12】要直接引用target字段，不能套两层`impl Deref`
        // * * ️📝会导致「临时变量引用」问题
        &mut self.inner.target
    }
}

/// 委托token
impl Budget for TaskLink {
    fn priority(&self) -> ShortFloat {
        self.token.priority()
    }

    fn __priority_mut(&mut self) -> &mut ShortFloat {
        self.token.__priority_mut()
    }

    fn durability(&self) -> ShortFloat {
        self.token.durability()
    }

    fn __durability_mut(&mut self) -> &mut ShortFloat {
        self.token.__durability_mut()
    }

    fn quality(&self) -> ShortFloat {
        self.token.quality()
    }

    fn __quality_mut(&mut self) -> &mut ShortFloat {
        self.token.__quality_mut()
    }
}

/// 委托token
impl Item for TaskLink {
    type Key = String;
    fn key(&self) -> &String {
        self.token.key()
    }
}

/// 委托inner
/// * ⚠️此处会对共享引用进行借用
impl TLink<Task> for TaskLink {
    fn target<'r, 's: 'r>(&'s self) -> impl Deref<Target = Task> + 'r {
        // ! 🚩【2024-06-22 12:21:12】要直接引用target字段，不能套两层`impl Deref`
        // * * ️📝会导致「临时变量引用」问题
        self.inner.target.get_()
    }

    fn target_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = Task> + 'r {
        // ! 🚩【2024-06-22 12:21:12】要直接引用target字段，不能套两层`impl Deref`
        // * * ️📝会导致「临时变量引用」问题
        self.inner.target.mut_()
    }

    fn link_type(&self) -> TLinkType {
        self.inner.link_type()
    }

    fn indexes(&self) -> &[usize] {
        self.inner.indexes()
    }
}

impl TaskLink {
    /// 🆕统一收归的「任务链记录长度」
    const RECORD_LENGTH: usize = DEFAULT_PARAMETERS.term_link_record_length;

    /// 完全构造函数
    /// * 📌其中的「链接目标」是共享引用
    fn new(
        target_rc: RCTask,
        budget: BudgetValue,
        link_type: TLinkType,
        indexes: impl Into<Box<[usize]>>,
        record_length: usize,
    ) -> Self {
        // * 🚩先生成Token
        let indexes = indexes.into();
        let key = Self::generate_key_for_task_link(&target_rc.get_(), link_type, &indexes);
        let token = Token::new(key, budget);
        // * 🚩再传入生成内部链接
        let inner = TLinkage::new_direct(target_rc, link_type, indexes);
        // * 🚩使用定长数组存储：统一默认值
        let recorded_links = vec![(String::default(), 0); record_length].into_boxed_slice();
        Self {
            inner,
            token,
            recorded_links,
            n_recorded_term_links: 0,
        }
    }

    /// 特别为任务链生成索引键
    fn generate_key_for_task_link(
        target: &Task,
        link_type: TLinkType,
        indexes: &[usize],
    ) -> String {
        let key = Self::generate_key_base(link_type, indexes);
        join! {
            => key
            => target.key()
        }
    }

    /// 🆕传递「链接记录长度」的默认值
    fn with_default_record_len(
        target_rc: RCTask,
        budget: BudgetValue,
        link_type: TLinkType,
        indexes: impl Into<Box<[usize]>>,
    ) -> Self {
        Self::new(target_rc, budget, link_type, indexes, Self::RECORD_LENGTH)
    }

    /// 从模板构建
    /// * 🚩【2024-06-05 01:05:16】唯二的公开构造函数（入口），基于「词项链模板」构造
    /// * 📝【2024-05-30 00:46:38】只在「链接概念到任务」中使用
    /// * 🚩【2024-06-22 12:37:45】此处使用默认长度构建
    pub fn from_template(
        target_rc: RCTask,
        template: &TermLinkTemplate,
        budget: BudgetValue,
    ) -> Self {
        let indexes = template.indexes().to_vec().into_boxed_slice();
        Self::with_default_record_len(target_rc, budget, template.link_type(), indexes)
    }

    /// 🆕专用于创建「自身」链接
    /// * 📝仅在「链接到任务」时被构造一次
    /// * 🎯用于推理中识别并分派
    /// * 🚩使用「SELF」类型，并使用空数组
    pub fn new_self(target_rc: RCTask) -> Self {
        // * 🚩预算值就是任务的预算值
        let target_ref = target_rc.get_();
        let budget = BudgetValue::from_other(&*target_ref);
        drop(target_ref); // 手动丢弃引用代理，解除对target_rc的借用

        // * 🚩空的索引（不需要）
        let indexes = vec![].into_boxed_slice();

        // * 🚩构造
        Self::with_default_record_len(target_rc, budget, TLinkType::SELF, indexes)
    }

    /// * 🎯用于从「新近任务袋」中获取「新近任务」：根据「新近」调配优先级
    /// * 📝在「概念推理」的「准备待推理词项链」的过程中用到
    /// * 🔗ProcessReason.chooseTermLinksToReason
    pub fn novel(&mut self, term_link: &TermLink, current_time: ClockTime) -> bool {
        // * 🚩重复目标⇒非新近
        {
            // * 📝此处需要销毁获得的引用代理（得手动管理生命周期）
            let b_term = &*term_link.target();
            let t_term = self.target();
            if b_term == t_term.content() {
                return false;
            }
        }
        // * 🚩检查所有已被记录的词项链
        let link_key = term_link.key();
        for i in 0..self.n_recorded_term_links {
            let existed_i = i % self.recorded_links.len();
            let (existed_key, existed_time) = &self.recorded_links[existed_i];
            // * 🚩重复key⇒检查时间
            if link_key == existed_key {
                // * 🚩并未足够「滞后」⇒非新近 | 💭或许是一种「短期记忆」的表示
                if current_time < existed_time + self.recorded_links.len() {
                    return false;
                }
                // * 🚩足够「滞后」⇒更新时间，判定为「新近」
                else {
                    self.recorded_links[existed_i].1 = current_time;
                    return true;
                }
            }
        }
        // * 🚩没检查到已有的：记录新匹配的词项链 | ️📝有可能覆盖
        let next = self.n_recorded_term_links % self.recorded_links.len();
        self.recorded_links[next] = (link_key.clone(), current_time);
        if self.n_recorded_term_links < self.recorded_links.len() {
            self.n_recorded_term_links += 1;
            // ? 💭只增不减？似乎会导致「信念固化」（or 始终覆盖最新的，旧的得不到修改）
        }
        true
    }
}

impl ToDisplayAndBrief for TaskLink {
    fn to_display(&self) -> String {
        join! {
            => self.token.budget_to_display()
            => " "
            => self.key()
            => " "
            => self.target().stamp_to_display()
        }
    }
}
