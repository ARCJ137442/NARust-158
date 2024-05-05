//! 🎯复刻OpenNARS `nars.entity.Task`
//! TODO: 着手开始复刻

use super::{BudgetValueConcrete, Item, Sentence, SentenceConcrete};
use crate::storage::BagKey;

/// 模拟OpenNARS `nars.entity.Task`
///
/// # 📄OpenNARS
///
/// A task to be processed, consists of a Sentence and a BudgetValue
pub trait Task {
    /// 绑定的「语句」类型
    ///
    /// ? 【2024-05-05 19:43:16】是要「直接绑定语句」还是「绑定真值、时间戳等，再由其组装成『语句』」
    /// * 🚩【2024-05-05 19:43:42】目前遵循「依赖封闭」的原则，暂还是使用「直接绑定语句」的方式
    type Sentence: SentenceConcrete;

    /// 绑定的「元素id」类型
    /// * 🎯用于实现[`Item`]
    type Key: BagKey;

    /// 绑定的「预算值」类型
    type Budget: BudgetValueConcrete;

    /// 🆕获取内部作为引用的「元素id」
    /// * 🎯用于返回引用而非值
    /// * 📌实现者可能需要在内部缓存一个「元素id」而非「直接从『语句』处获取」
    /// * 📌可用于对接[`Sentence::to_key`]
    fn __key(&self) -> &Self::Key;

    /// 模拟`Task.sentence`、`Task.getSentence`
    fn sentence(&self) -> &Self::Sentence;
    /// 🆕[`Task::sentence`]的可变版本
    /// * 🎯用于自动实现[`Sentence`]
    fn sentence_mut(&mut self) -> &mut Self::Sentence;

    /// 模拟`Task.budget`、`Task.getBudget`
    fn budget(&self) -> &Self::Budget;
    /// 🆕[`Task::budget`]的可变版本
    /// * 🎯用于自动实现[`super::BudgetValue`]
    fn budget_mut(&mut self) -> &mut Self::Budget;
}

/// 自动实现「语句」
/// * ✅同时自动实现「时间戳」[`super::Stamp`]
impl<T: Task> Sentence for T {
    type Truth = <<Self as Task>::Sentence as Sentence>::Truth;
    type Stamp = <<Self as Task>::Sentence as Sentence>::Stamp;

    #[inline(always)]
    fn content(&self) -> &crate::language::Term {
        self.sentence().content()
    }

    #[inline(always)]
    fn content_mut(&mut self) -> &mut crate::language::Term {
        self.sentence_mut().content_mut()
    }

    #[inline(always)]
    fn punctuation(&self) -> &super::SentenceType<Self::Truth> {
        self.sentence().punctuation()
    }

    #[inline(always)]
    fn punctuation_mut(&mut self) -> &mut super::SentenceType<Self::Truth> {
        self.sentence_mut().punctuation_mut()
    }

    #[inline(always)]
    fn stamp(&self) -> &Self::Stamp {
        self.sentence().stamp()
    }

    #[inline(always)]
    fn stamp_mut(&mut self) -> &mut Self::Stamp {
        self.sentence_mut().stamp_mut()
    }

    #[inline(always)]
    fn revisable(&self) -> bool {
        self.sentence().revisable()
    }

    #[inline(always)]
    fn revisable_mut(&mut self) -> &mut bool {
        self.sentence_mut().revisable_mut()
    }
}

/// 自动实现「Item」
/// * ✅同时自动实现「预算值」[`super::BudgetValue`]
impl<T: Task> Item for T {
    type Key = <Self as Task>::Key;
    type Budget = <Self as Task>::Budget;

    #[inline(always)]
    fn key(&self) -> &Self::Key {
        self.__key()
    }

    #[inline(always)]
    fn budget(&self) -> &Self::Budget {
        self.budget()
    }

    #[inline(always)]
    fn budget_mut(&mut self) -> &mut Self::Budget {
        self.budget_mut()
    }
}

/// 初代实现
mod impl_v1 {
    use super::*;

    /// [`Task`]的初代实现
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TaskV1<S, K, B>
    where
        S: SentenceConcrete,
        K: BagKey,
        B: BudgetValueConcrete,
    {
        sentence: S,
        key: K,
        budget: B,
    }

    /// 直接实现
    impl<S, K, B> Task for TaskV1<S, K, B>
    where
        S: SentenceConcrete,
        K: BagKey,
        B: BudgetValueConcrete,
    {
        type Sentence = S;
        type Key = K;
        type Budget = B;

        #[inline(always)]
        fn __key(&self) -> &Self::Key {
            &self.key
        }

        #[inline(always)]
        fn sentence(&self) -> &Self::Sentence {
            &self.sentence
        }

        #[inline(always)]
        fn sentence_mut(&mut self) -> &mut Self::Sentence {
            &mut self.sentence
        }

        #[inline(always)]
        fn budget(&self) -> &Self::Budget {
            &self.budget
        }

        #[inline(always)]
        fn budget_mut(&mut self) -> &mut Self::Budget {
            &mut self.budget
        }
    }
}
pub use impl_v1::*;
/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
