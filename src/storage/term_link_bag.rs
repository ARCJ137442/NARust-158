//! 🎯复刻OpenNARS `nars.entity.TermLinkBag`
//! * 📌「词项链袋」
//! * ✅【2024-05-04 17:50:50】基本功能复刻完成
//! * ✅【2024-05-06 00:13:38】初代实现完成

use super::BagConcrete;
use crate::{
    entity::{Item, TaskLink, TermLinkConcrete},
    global::ClockTime,
    nars::DEFAULT_PARAMETERS,
};

/// 模拟`nars.entity.TermLinkBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+词项链+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
/// * ⚠️ 在[「袋」](Bag)的基础上，对[「取出」](Bag::take_out)做了优化
///   * 🎯优化目的：避免重复推理
/// * 🚩【2024-05-07 20:57:36】锁定是「具体特征」
///   * 📌目前必须有构造函数
///   * ⚠️不然会有`ConceptBag: BagConcrete<Self::Concept> + ConceptBag`的「双重叠加」问题
///     * ❌这样会出现两套实现
pub trait TermLinkBag: BagConcrete<Self::Link> {
    /// 绑定的「词项链」类型
    /// * 🎯一种实现只能对应一种「词项链袋」
    type Link: TermLinkConcrete;

    /// 结合已有的「任务链」和「时间」去取出
    ///
    /// TODO: 关于`task_link`的可变问题，有待在[`TermLink::novel`]中修复
    ///
    /// # 📄OpenNARS
    ///
    /// Replace default to prevent repeated inference, by checking TermLink
    ///
    /// @param taskLink The selected TermLink
    /// @param time     The current time
    /// @return The selected TermLink
    fn take_out_with_link<LTaskLink>(
        &mut self,
        task_link: &mut LTaskLink,
        time: ClockTime,
    ) -> Option<Self::Link>
    where
        LTaskLink: TaskLink<Budget = <Self::Link as Item>::Budget, Key = <Self::Link as Item>::Key>,
    {
        /* 📄OpenNARS源码：
        for (int i = 0; i < Parameters.MAX_MATCHED_TERM_LINK; i++) {
            TermLink termLink = takeOut();
            if (termLink == null) {
                return null;
            }
            if (taskLink.novel(termLink, time)) {
                return termLink;
            }
            putBack(termLink);
        }
        return null; */
        for _ in 0..DEFAULT_PARAMETERS.max_matched_term_link {
            match self.take_out() {
                None => return None,
                Some(term_link) => {
                    if task_link.update_novel(&term_link, time) {
                        return Some(term_link);
                    }
                    self.put_back(term_link);
                }
            }
        }
        None
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::{
        entity::{BudgetV1, SentenceV1, StampV1, TaskV1, TermLinkV1, TruthV1},
        storage::{BagKeyV1, BagV1},
    };

    /// 自动为「任务链+[`BagKeyV1`]+[`BagV1`]」实现「词项链袋」
    impl<T: TermLinkConcrete<Key = BagKeyV1>> TermLinkBag for BagV1<T> {
        type Link = T;
    }

    /// 初代[`TermLinkBag`]实现
    /// * 🚩【2024-05-05 22:29:47】只需限定一系列类型，而无需再声明新`struct`
    pub type TermLinkBagV1 =
        BagV1<TermLinkV1<TaskV1<SentenceV1<TruthV1, StampV1>, BagKeyV1, BudgetV1>>>;
}
pub use impl_v1::*;

// * ✅单元测试参见`super::Bag`
