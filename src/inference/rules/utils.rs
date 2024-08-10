//! 「规则表」中的「实用定义」
//! * 🎯用于辅助理解的工具性定义

use crate::{
    entity::StatementPosition,
    language::{CompoundTerm, Statement, Term},
};

/// 在断言的情况下，从[`Term`]中提取[`CompoundTerm`]
/// * 🎯对标OpenNARS`(CompoundTerm) term`的转换
pub fn cast_compound(term: Term) -> CompoundTerm {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_compound(),
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 在断言的情况下，从[`Term`]中提取[`Statement`]
/// * 🎯对标OpenNARS`(Statement) term`的转换
pub fn cast_statement(term: Term) -> Statement {
    // * 🚩调试时假定复合词项
    debug_assert!(
        term.is_statement(),
        "强制转换失败：词项\"{term}\"必须是陈述"
    );
    term.try_into().expect("必定是陈述")
}

/// * 📌主要包含两项：对称项/反对称项
///   * 可能有「恒等项」满足「取反等于自身」
///   * 📄见[`SyllogismSide`]
/// * 🚩基础行为：两类项可以相互转换——[取反](Dual::opposite)算子
///   * 对称项⇒反对称项
///   * 反对称项⇒对称项
///   * 恒等项⇒自身
pub trait Opposite: Sized {
    /// 调转到「相反方向」「相反位置」
    /// * 🎯抽象自各个「三段论位置」
    /// * 🎯为「三段论图式」添加方法
    /// * 🚩具体步骤
    ///   * 对称项 ⇒ 反对称项
    ///   * 反对称项 ⇒ 对称项
    fn opposite(self) -> Self;

    /// 返回自身与「自身的相反位置」
    ///   * 对称项 ⇒ [对称项, 反对称项]
    ///   * 反对称项 ⇒ [反对称项, 对称项]
    fn and_opposite(self) -> [Self; 2]
    where
        Self: Clone,
    {
        [self.clone(), self.opposite()]
    }
}

/// 统一表示所有「两项中选取一项，或可将其调换位置」的行为
/// * 📌包含两项：对称项/反对称项
/// * 🚩基础行为：在「左右」两项中选择
///   * 其中的「对称项」⇒选择二者中的前一个，并且不改变顺序
///   * 其中的「反对称项」⇒选择二者中的后一个，并且交换顺序
pub trait Select {
    /// 根据「对称性/反对称性」
    fn select<T>(&self, left_right: [T; 2]) -> [T; 2];

    /// 选择某一个「对称项」
    /// * 📝此时「对称项」「反对称项」分别充当「左」「右」两个角色
    ///   * 对称项 ⇒ 前一个
    ///   * 反对称项 ⇒ 后一个
    fn select_one<T>(&self, left_right: [T; 2]) -> T {
        let [selected, _] = self.select(left_right);
        selected
    }

    /// 选择另一个「对称项」
    /// * 📝此时「对称项」「反对称项」将反向选择
    ///   * 对称项 ⇒ 后一个
    ///   * 反对称项 ⇒ 前一个
    fn select_another<T>(&self, left_right: [T; 2]) -> T {
        let [_, selected] = self.select(left_right);
        selected
    }
}

/// 为布尔值实现「选择」
/// * 🎯简化类似「转换顺序」的模式匹配
/// * 💭【2024-08-07 23:45:41】在不明确布尔值意义如「是否切换」的情况下 慎用
impl Select for bool {
    /// 为布尔值[`bool`]特别实现的「选择」
    /// * 📌`true`总是会在[`select_one`]`([false_side, true_side])`中选中`true_side`
    ///   * ✅`false`亦然
    /// * 📌交换规则
    ///   * `false` => 不交换（对称）
    ///   * `true` => 交换（反对称）
    fn select<T>(&self, [false_side, true_side]: [T; 2]) -> [T; 2] {
        match self {
            true => [true_side, false_side],
            false => [false_side, true_side],
        }
    }
}

/// 记录各处推理中「前提」的位置
/// * 🎯标记诸如「复合词项来自信念」等
/// * 📄例如
///   * 任务
///   * 信念
/// * 📌二者形成「对偶关系」
///   * 对称项：[`Self::Task`]
///   * 反对称项：[`Self::Belief`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PremiseSource {
    /// 任务
    Task,
    /// 信念
    Belief,
}

impl Select for PremiseSource {
    /// 在「任务」「信念」中选择
    /// * 📌选取原则：**根据内容选中的**永远在**第一个**
    /// * 🚩传入`[任务, 信念]`，始终返回`[任务/信念, 信念/任务]`
    ///   * 「任务」 ⇒ `[任务, 信念]`
    ///   * 「信念」 ⇒ `[信念, 任务]`
    /// * ✅【2024-08-01 21:27:43】正向选择、反向选择可直接`let [X, _] = ...`与`let [_, X] = ...`搞定
    ///   * 📌【2024-08-01 21:28:22】无需「选择反转」
    fn select<T>(&self, [task_thing, belief_thing]: [T; 2]) -> [T; 2] {
        use PremiseSource::*;
        match self {
            Task => [task_thing, belief_thing],
            Belief => [belief_thing, task_thing],
        }
    }
}

impl Opposite for StatementPosition {
    /// 🆕调转到相反位置
    fn opposite(self) -> Self {
        match self {
            Subject => Predicate,
            Predicate => Subject,
        }
    }
}

impl Select for StatementPosition {
    /// 根据「三段论位置」从参数中选取一个参数
    /// * 🎯在「陈述选择」的过程中使用，同时需要前后两项
    /// * 🚩数组的第一项即为「选中项」
    fn select<T>(&self, [subject, predicate]: [T; 2]) -> [T; 2] {
        match self {
            Subject => [subject, predicate],
            Predicate => [predicate, subject],
        }
    }
}

/// 三段论图式
/// * 🎯模拟「三段论推理」中「公共项在两陈述的位置」的四种情况
/// * 📝左边任务（待处理），右边信念（已接纳）
/// * 🚩公共词项在两个陈述之中的顺序
/// * 🚩使用二元组实现，允许更细化的组合
///   * ✨基本等同于整数（低开销）类型
/// * 🚩【2024-07-12 21:17:33】现在改为二元数组
///   * 💭相同的效果，更简的表达
///   * 📌相同类型的序列，宜用数组表达
/// * 📝四种主要情况：
///   * 主项-主项
///   * 主项-谓项
///   * 谓项-主项
///   * 谓项-谓项
///
/// # 📄OpenNARS
///
/// location of the shared term
pub type SyllogismFigure = [StatementPosition; 2];

impl Opposite for SyllogismFigure {
    /// 🆕调转到相反位置：内部俩均如此
    fn opposite(self) -> Self {
        let [subject, predicate] = self;
        [subject.opposite(), predicate.opposite()]
    }
}

impl StatementPosition {
    /// 🆕构造「三段论图式」
    /// * 🎯[`RuleTables::__index_to_figure`]
    /// * 🚩直接构造二元组
    pub fn build_figure(self, other: Self) -> SyllogismFigure {
        [self, other]
    }
}
use StatementPosition::*;

/// 存储「三段论图式」常量
/// * 🎯可完全引用，可简短使用
///   * ⚡长度与OpenNARS的`11`、`12`相近
/// * 🚩仅四种
pub mod syllogistic_figures {
    use super::*;

    /// [三段论图式](SyllogismFigure)/常用/主项-主项
    #[doc(alias = "SUBJECT_SUBJECT")]
    pub const SS: SyllogismFigure = [Subject, Subject];

    /// [三段论图式](SyllogismFigure)/常用/主项-谓项
    #[doc(alias = "SUBJECT_PREDICATE")]
    pub const SP: SyllogismFigure = [Subject, Predicate];

    /// [三段论图式](SyllogismFigure)/常用/谓项-主项
    #[doc(alias = "PREDICATE_SUBJECT")]
    pub const PS: SyllogismFigure = [Predicate, Subject];

    /// [三段论图式](SyllogismFigure)/常用/谓项-谓项
    #[doc(alias = "PREDICATE_PREDICATE")]
    pub const PP: SyllogismFigure = [Predicate, Predicate];
}

/// 三段论推理中的「某侧」
/// * 📌包含「主项/谓项/整个词项」
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyllogismSide {
    /// 主项（第一项）
    Subject = 0,
    /// 谓项（第二项）
    Predicate = 1,
    /// 整个词项（整体）
    Whole = -1,
}

impl SyllogismSide {
    /// 🆕从可用的「数组索引」中来
    /// * 🚩核心：Some(0)→主项，Some(1)→谓项，None→整体`<主项 --> 谓项>`
    pub fn from_index(index: Option<usize>) -> Self {
        use SyllogismSide::*;
        match index {
            Some(0) => Subject,
            Some(1) => Predicate,
            None => Whole,
            _ => panic!("无效索引"),
        }
    }

    /* /// 尝试以此「选择」一个词项
    /// * 🚩主项/谓项⇒尝试as为一个陈述并选择之
    /// * 🚩整体⇒返回`Some(自身)`
    /// * 📌【2024-08-04 23:56:16】目前仅选择「陈述引用」
    pub fn select(self, term: &Term) -> Option<&Term> {
        use SyllogismSide::*;
        match self {
            Subject => term.as_statement().map(|s| s.subject),
            Predicate => term.as_statement().map(|s| s.predicate),
            Whole => Some(term),
        }
    } */

    /// 互斥性选择
    /// * 🚩主项/谓项⇒尝试as为一个陈述并选择之，返回 `[谓项,主项]`/`[谓项,主项]`
    /// * 🚩整体⇒返回`[Some(自身), None]`
    /// * 📌【2024-08-04 23:56:16】目前仅选择「陈述引用」
    /// * 🎯在「条件三段论」中选择陈述组分
    pub fn select_exclusive(self, term: &Term) -> [Option<&Term>; 2] {
        use SyllogismSide::*;
        match (self, term.as_statement()) {
            (Subject, Some(s)) => [Some(s.subject), Some(s.predicate)], // 互斥性引用
            (Predicate, Some(s)) => [Some(s.predicate), Some(s.subject)], // 互斥性引用
            (Whole, _) => [Some(term), None],                           // 整体⇒聚集于一处
            _ => [None, None],                                          // 无效情况
        }
    }
}

/// 从「三段论位置」到「三段论某侧」
/// * 📝兼容性转换
impl From<StatementPosition> for SyllogismSide {
    fn from(value: StatementPosition) -> Self {
        match value {
            Subject => Self::Subject,
            Predicate => Self::Predicate,
        }
    }
}

impl Opposite for SyllogismSide {
    /// 🆕调转到相反位置
    fn opposite(self) -> Self {
        use SyllogismSide::*;
        match self {
            Subject => Predicate,
            Predicate => Subject,
            Whole => Whole, // * 📌整体反过来还是整体
        }
    }
}

// ! ℹ️【2024-08-05 18:47:31】有关「辅助测试用代码」如「预期测试宏」均放到`inference`的根模块下
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Symmetry {
        Symmetric = 0,
        Asymmetric = 1,
    }
    use nar_dev_utils::asserts;
    use Symmetry::*;

    impl Opposite for Symmetry {
        fn opposite(self) -> Self {
            match self {
                Symmetric => Asymmetric,
                Asymmetric => Symmetric,
            }
        }
    }

    impl Select for Symmetry {
        fn select<T>(&self, [left, right]: [T; 2]) -> [T; 2] {
            match self {
                Symmetric => [left, right],
                Asymmetric => [right, left],
            }
        }
    }

    #[test]
    fn test_opposite() {
        asserts! {
            Symmetric.opposite() => Asymmetric,
            Asymmetric.opposite() => Symmetric,
        }
    }

    #[test]
    fn test_select() {
        asserts! {
            Symmetric.select([0, 1]) => [0, 1],
            Asymmetric.select([0, 1]) => [1, 0],
            Symmetric.select(["0", "1"]) => ["0", "1"],
            Asymmetric.select(["0", "1"]) => ["1", "0"],
        }
    }
}
