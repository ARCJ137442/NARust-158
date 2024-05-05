//! 🎯复刻OpenNARS `nars.entity.Task`
//! TODO: 着手开始复刻

use super::{BudgetValueConcrete, Item, SentenceConcrete};

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

    /// 绑定的「预算值」类型
    type Budget: BudgetValueConcrete;

    fn sentence(&self) -> &Self::Sentence;
    fn budget(&self) -> &Self::Budget;
}

// 自动实现「Item」
// impl<T: Task> Item for T {
//     type Key = String; // TODO: 有待解耦

//     type Budget = <Self as Task>::Budget;

//     fn key(&self) -> &Self::Key {
//         self.sentence().to_key()
//     }

//     fn budget(&self) -> &Self::Budget {
//         todo!()
//     }

//     fn budget_mut(&mut self) -> &mut Self::Budget {
//         todo!()
//     }
// }
