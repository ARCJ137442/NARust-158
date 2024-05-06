//! 🎯复刻OpenNARS `nars.entity.Concept`
//! TODO: 着手开始复刻

use super::{Item, Sentence, Stamp, Task, TaskLink, TermLinkConcrete, TruthValue};
use crate::{
    language::Term,
    storage::{TaskLinkBag, TermLinkBag},
};

/// 模拟OpenNARS `nars.entity.Concept`
/// * 🚩【2024-05-04 17:28:30】「概念」首先能被作为「Item」使用
pub trait Concept: Item {
    /// 绑定的「时间戳」类型
    type Stamp: Stamp;

    /// 绑定的「真值」类型
    type Truth: TruthValue;

    /// 模拟`Concept.term`、`Concept.getTerm`
    /// * 🚩只读：OpenNARS仅在构造函数中赋值
    ///
    /// # 📄OpenNARS
    ///
    /// ## `term`
    ///
    /// The term is the unique ID of the concept
    ///
    /// ## `getTerm`
    ///
    /// Return the associated term, called from Memory only
    ///
    /// @return The associated term
    fn term(&self) -> &Term;

    /// 模拟`Concept.taskLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Task links for indirect processing
    fn __task_links<S, T, Link>(&self) -> &impl TaskLinkBag<Link = Link>
    where
        // 从「语句」到「任务」再到「任务链袋」
        S: Sentence<Truth = Self::Truth, Stamp = Self::Stamp>,
        T: Task<Sentence = S, Key = Self::Key, Budget = Self::Budget>,
        Link: TaskLink<Key = Self::Key, Budget = Self::Budget, Task = T>;
    /// [`Concept::__task_links`]的可变版本
    fn __task_links_mut<S, T, Link>(&mut self) -> &mut impl TaskLinkBag<Link = Link>
    where
        // 从「语句」到「任务」再到「任务链袋」
        S: Sentence<Truth = Self::Truth, Stamp = Self::Stamp>,
        T: Task<Sentence = S, Key = Self::Key, Budget = Self::Budget>,
        Link: TaskLink<Key = Self::Key, Budget = Self::Budget, Task = T>;

    /// 模拟`Concept.termLinks`
    /// * 🚩私有：未对外暴露直接的公开接口
    ///
    /// # 📄OpenNARS
    ///
    /// Term links between the term and its components and compounds
    fn __term_links<S, T, Link>(&self) -> &impl TermLinkBag<Link = Link>
    where
        Link: TermLinkConcrete<Key = Self::Key, Budget = Self::Budget>;
    /// [`Concept::__term_links`]的可变版本
    fn __term_links_mut<S, T, Link>(&mut self) -> &mut impl TermLinkBag<Link = Link>
    where
        Link: TermLinkConcrete<Key = Self::Key, Budget = Self::Budget>;
}

/// TODO: 初代实现
mod impl_v1 {
    use super::*;
}
pub use impl_v1::*;

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
