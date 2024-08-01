//! 「规则表」中的「实用定义」
//! * 🎯用于辅助理解的工具性定义

use crate::language::{CompoundTerm, Statement, StatementRef, Term};

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
        "强制转换失败：词项\"{term}\"必须是复合词项"
    );
    term.try_into().expect("必定是复合词项")
}

/// 记录各处推理中「前提」的位置
/// * 🎯标记诸如「复合词项来自信念」等
/// * 📄例如
///   * 任务
///   * 信念
#[derive(Debug, Clone, Copy)]
pub enum PremiseSource {
    /// 任务
    Task,
    /// 信念
    Belief,
}

impl PremiseSource {
    /// 交换「任务⇄信念」
    pub fn swap(self) -> Self {
        use PremiseSource::*;
        match self {
            Task => Belief,
            Belief => Task,
        }
    }

    /// 在「任务」「信念」中选择
    /// * 🚩传入`[任务, 信念]`，始终返回`[任务/信念, 信念/任务]`
    ///   * 「任务」 ⇒ `[任务, 信念]`
    ///   * 「信念」 ⇒ `[信念, 任务]`
    pub fn select<T>(self, [task_thing, belief_thing]: [T; 2]) -> [T; 2] {
        use PremiseSource::*;
        match self {
            Task => [task_thing, belief_thing],
            Belief => [belief_thing, task_thing],
        }
    }
}

pub trait Opposite {
    /// 调转到「相反方向」「相反位置」
    /// * 🎯抽象自各个「三段论位置」
    /// * 🎯为「三段论图式」添加方法
    fn opposite(self) -> Self;

    /// 返回自身与「自身的相反位置」
    fn and_opposite(self) -> [Self; 2]
    where
        Self: Clone,
    {
        [self.clone(), self.opposite()]
    }
}

/// 🆕三段论位置
/// * 🎯用于表征[`RuleTables::index_to_figure`]推导出的「三段论子类型」
/// * 📝OpenNARS中是在「三段论推理」的「陈述🆚陈述」中表示「位置关系」
///   * 📄`<A --> B>`与`<B --> C>`中，`B`就分别在`1`、`0`两个索引位置
///     * 📌因此有`SP`或`Subject-Predicate`
///     * 📌同时也有了其它三种「三段论图式」
/// * 🚩两种情况：
///   * 主项
///   * 谓项
#[doc(alias = "SyllogismLocation")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyllogismPosition {
    /// 主项（第一项）
    Subject = 0,
    /// 谓项（第二项）
    Predicate = 1,
}

impl Opposite for SyllogismPosition {
    /// 🆕调转到相反位置
    fn opposite(self) -> Self {
        match self {
            Subject => Predicate,
            Predicate => Subject,
        }
    }
}

impl SyllogismPosition {
    /// 🆕从「数组索引」中来
    /// * 🎯[`RuleTables::__index_to_figure`]
    /// * 🚩核心：0→主项，1→谓项，整体`<主项 --> 谓项>`
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Subject,
            1 => Predicate,
            _ => panic!("无效索引"),
        }
    }

    /// 🆕构造「三段论图式」
    /// * 🎯[`RuleTables::__index_to_figure`]
    /// * 🚩直接构造二元组
    pub fn build_figure(self, other: Self) -> SyllogismFigure {
        [self, other]
    }

    /// 根据「三段论位置」从参数中选取一个参数
    /// * 🎯在「陈述选择」的过程中使用，同时需要前后两项
    /// * 🚩数组的第一项即为「选中项」
    pub fn select_and_other<T>(self, [subject, predicate]: [T; 2]) -> [T; 2] {
        match self {
            Subject => [subject, predicate],
            Predicate => [predicate, subject],
        }
    }

    /// 根据「三段论位置」从参数中选取一个参数
    /// * 🎯在「陈述解包」的过程中使用
    pub fn select<T>(self, sub_pre: [T; 2]) -> T {
        let [selected, _] = self.select_and_other(sub_pre);
        selected
    }
}
use SyllogismPosition::*;

/// 以此扩展到「陈述」的功能
impl StatementRef<'_> {
    /// 根据「三段论位置」扩展获取「三段论位置」对应的「词项」
    pub fn get_at_position(&self, position: SyllogismPosition) -> &Term {
        match position {
            Subject => self.subject(),
            Predicate => self.predicate(),
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
pub type SyllogismFigure = [SyllogismPosition; 2];

impl Opposite for SyllogismFigure {
    /// 🆕调转到相反位置：内部俩均如此
    fn opposite(self) -> Self {
        let [subject, predicate] = self;
        [subject.opposite(), predicate.opposite()]
    }
}

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
