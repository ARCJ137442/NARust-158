//! 🎯复刻OpenNARS `nars.entity.TaskLink`
//! TODO: 着手开始复刻

use crate::global::ClockTime;

use super::TermLink;

/// 模拟OpenNARS `nars.entity.TaskLink`
pub trait TaskLink: TermLink {
    /// * 🚩【2024-05-04 17:44:14】从「词项链袋」中来，临时占坑
    ///
    /// # 📄OpenNARS
    ///
    /// To check whether a TaskLink should use a TermLink, return false if they
    /// interacted recently
    ///
    /// called in TermLinkBag only
    ///
    /// @param termLink    The TermLink to be checked
    /// @param currentTime The current time
    /// @return Whether they are novel to each other
    fn novel(&self, term_link: &impl TermLink, time: ClockTime) -> bool;
}
