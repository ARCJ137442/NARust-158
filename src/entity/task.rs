//! 🎯复刻OpenNARS `nars.entity.Task`

use super::{BudgetValueConcrete, Item, Sentence, SentenceConcrete};
use crate::{global::RC, storage::BagKey};
use std::hash::Hash;

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
    ///
    /// # 📄OpenNARS
    ///
    /// The sentence of the Task
    fn sentence(&self) -> &Self::Sentence;
    /// 🆕[`Task::sentence`]的可变版本
    /// * 🎯用于自动实现[`Sentence`]
    fn sentence_mut(&mut self) -> &mut Self::Sentence;

    /// 模拟`Task.budget`、`Task.getBudget`
    /// * 📝OpenNARS中的`Task`直接从`Item`中拿到了`Budget`字段
    ///   * 此处为避免与[`Item::budget`]命名冲突，采用内部化命名
    fn __budget(&self) -> &Self::Budget;
    /// 🆕[`Task::budget`]的可变版本
    /// * 🎯用于自动实现[`super::BudgetValue`]
    fn __budget_mut(&mut self) -> &mut Self::Budget;

    /// 模拟`Task.parentTask`、`Task.getParentTask`
    /// * 🚩【2024-05-05 20:51:48】目前对「共享引用」使用「引用计数」处理
    ///
    /// # 📄OpenNARS
    ///
    /// Task from which the Task is derived, or null if input
    fn parent_task(&self) -> &Option<RC<Self>>;
    /// [`Task::parent_task`]的可变版本
    /// * 📌只能修改「指向哪个[`Task`]」，不能修改所指向[`Task`]内部的数据
    fn parent_task_mut(&mut self) -> &mut Option<RC<Self>>;

    /// 模拟`Task.parentBelief`、`Task.getParentBelief`
    /// * 🚩【2024-05-05 20:51:48】目前对「共享引用」使用「引用计数」处理
    ///
    /// # 📄OpenNARS
    ///
    /// Belief from which the Task is derived, or null if derived from a theorem
    fn parent_belief(&self) -> &Option<RC<Self::Sentence>>;
    /// [`Task::parent_belief`]的可变版本
    /// * 📌只能修改「指向哪个[`Sentence`]」，不能修改所指向[`Sentence`]内部的数据
    fn parent_belief_mut(&mut self) -> &mut Option<RC<Self::Sentence>>;

    /// 模拟`Task.bestSolution`
    /// * 🚩【2024-05-05 20:51:48】目前对「共享引用」使用「引用计数」处理
    ///
    /// # 📄OpenNARS
    ///
    /// For Question and Goal: best solution found so far
    fn best_solution(&self) -> &Option<RC<Self::Sentence>>;
    /// [`Task::best_solution`]的可变版本
    /// * 📌只能修改「指向哪个[`Sentence`]」，不能修改所指向[`Sentence`]内部的数据
    fn best_solution_mut(&mut self) -> &mut Option<RC<Self::Sentence>>;
}

pub trait TaskConcrete: Task + Sized {
    /// 模拟`new Task(Sentence s, BudgetValue b, Task parentTask, Sentence parentBelief, Sentence solution)`
    /// * 🚩完全参数的构造函数
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for an activated task
    ///
    /// @param s            The sentence
    /// @param b            The budget
    /// @param parentTask   The task from which this new task is derived
    /// @param parentBelief The belief from which this new task is derived
    /// @param solution     The belief to be used in future inference
    fn __new(
        s: Self::Sentence,
        b: Self::Budget,
        parent_task: Option<RC<Self>>,
        parent_belief: Option<RC<Self::Sentence>>,
        solution: Option<RC<Self::Sentence>>,
    ) -> Self;

    /// 模拟`new Task(Sentence s, BudgetValue b)`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for input task
    ///
    /// @param s The sentence
    /// @param b The budget
    #[inline(always)]
    fn from_input(s: Self::Sentence, b: Self::Budget) -> Self {
        Self::__new(s, b, None, None, None)
    }

    /// 模拟`new Task(Sentence s, BudgetValue b, Task parentTask, Sentence parentBelief)`
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for a derived task
    ///
    /// @param s            The sentence
    /// @param b            The budget
    /// @param parentTask   The task from which this new task is derived
    /// @param parentBelief The belief from which this new task is derived
    #[inline(always)]
    fn from_derive(
        s: Self::Sentence,
        b: Self::Budget,
        parent_task: Option<RC<Self>>,
        parent_belief: Option<RC<Self::Sentence>>,
    ) -> Self {
        Self::__new(s, b, parent_task, parent_belief, None)
    }
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
        self.__budget()
    }

    #[inline(always)]
    fn budget_mut(&mut self) -> &mut Self::Budget {
        self.__budget_mut()
    }
}

/// 初代实现
mod impl_v1 {
    use std::fmt::Debug;

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
        parent_task: Option<RC<Self>>,
        parent_belief: Option<RC<S>>,
        best_solution: Option<RC<S>>,
    }

    /// 逐个字段实现
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
        fn __budget(&self) -> &Self::Budget {
            &self.budget
        }

        #[inline(always)]
        fn __budget_mut(&mut self) -> &mut Self::Budget {
            &mut self.budget
        }

        #[inline(always)]
        fn parent_task(&self) -> &Option<RC<Self>> {
            &self.parent_task
        }

        #[inline(always)]
        fn parent_task_mut(&mut self) -> &mut Option<RC<Self>> {
            &mut self.parent_task
        }

        #[inline(always)]
        fn parent_belief(&self) -> &Option<RC<Self::Sentence>> {
            &self.parent_belief
        }

        #[inline(always)]
        fn parent_belief_mut(&mut self) -> &mut Option<RC<Self::Sentence>> {
            &mut self.parent_belief
        }

        #[inline(always)]
        fn best_solution(&self) -> &Option<RC<Self::Sentence>> {
            &self.best_solution
        }

        #[inline(always)]
        fn best_solution_mut(&mut self) -> &mut Option<RC<Self::Sentence>> {
            &mut self.best_solution
        }
    }

    /// 直接实现
    impl<S, B> TaskConcrete for TaskV1<S, String, B>
    where
        S: SentenceConcrete,
        B: BudgetValueConcrete,
        S::Truth: Debug,
    {
        fn __new(
            s: Self::Sentence,
            b: Self::Budget,
            parent_task: Option<RC<Self>>,
            parent_belief: Option<RC<Self::Sentence>>,
            solution: Option<RC<Self::Sentence>>,
        ) -> Self {
            let key = s.to_key();
            Self {
                sentence: s,
                key,
                budget: b,
                parent_task,
                parent_belief,
                best_solution: solution,
            }
        }
    }
}
pub use impl_v1::*;
/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
