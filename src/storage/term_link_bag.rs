//! 🎯复刻OpenNARS `nars.entity.TermLinkBag`
//! * 📌「词项链袋」
//! * ✅【2024-05-04 17:50:50】基本功能复刻完成

use super::Bag;
use crate::{
    entity::{TaskLink, TermLink},
    global::ClockTime,
    nars::DEFAULT_PARAMETERS,
};

/// 模拟OpenNARS `nars.entity.TermLinkBag`
/// * 📌【2024-05-04 17:30:35】实际上就是「袋+词项链+特定参数」
///   * 📌目前不限制构造过程（即 不覆盖方法）
/// * 🚩有关「固定容量」与「遗忘时长」交给构造时决定
///   * ✅这也能避免冗余的对「记忆区」的引用
/// * ⚠️ 在[「袋」](Bag)的基础上，对[「取出」](Bag::take_out)做了优化
///   * 🎯优化目的：避免重复推理
pub trait TermLinkBag<L: TermLink>: Bag<L> {
    /// 结合已有的「任务链」和「时间」去取出
    ///
    /// # 📄OpenNARS
    ///
    /// Replace default to prevent repeated inference, by checking TaskLink
    ///
    /// @param taskLink The selected TaskLink
    /// @param time     The current time
    /// @return The selected TermLink
    fn take_out_with_link(&mut self, task_link: &impl TaskLink, time: ClockTime) -> Option<L> {
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
                    if task_link.novel(&term_link, time) {
                        return Some(term_link);
                    }
                    self.put_back(term_link);
                }
            }
        }
        None
    }
}
