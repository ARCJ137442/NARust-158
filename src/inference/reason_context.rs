//! 推理上下文
//! * 🎯【2024-05-06 22:26:56】最初用于解决「推理函数为『统一各参数的类参类型』被迫引入大量泛型参数与约束」的代码量膨胀问题
//! * 📝解决方法：
//!   * 一个[「推理上下文」](ReasonContext)作为「关联类型」定义在一个基础的「上下文」特征中，统一所用类型
//!   * 随后用「自动实现的超特征」为其添加方法

use crate::{
    entity::{
        BudgetValueConcrete, ConceptConcrete, SentenceConcrete, ShortFloat, StampConcrete,
        TaskConcrete, TaskLinkConcrete, TermLinkConcrete, TruthValueConcrete,
    },
    language::Term,
    storage::{BagKey, ConceptBag, MemoryConcrete, TaskLinkBag, TermLinkBag},
};

/// 🆕「推理上下文」
/// * 🎯【2024-05-06 22:16:22】最初用于提供「已被确定的类型约束」
///   * 📌避免过多函数中「泛型约束满天飞」并且「无法用宏简化」的场面
///     * 📝Rust中的宏并不能用在任何「可扩展为标签树」的地方
/// * 🚩【2024-05-07 19:06:48】只提供一系列关联类型，而暂不提供具体方法
///   * 这些「具体方法」留给后续的「自动实现之派生特征」，作为「追加方法」的手段
pub trait ReasonContext {
    // * 这下边都是为了「统一类型」 * //

    // 短浮点 → 真值 × 时间戳 → 语句 //

    /// 短浮点
    type ShortFloat: ShortFloat;

    /// 真值
    type Truth: TruthValueConcrete<E = Self::ShortFloat>;

    /// 时间戳
    type Stamp: StampConcrete;

    /// 语句
    type Sentence: SentenceConcrete<Truth = Self::Truth, Stamp = Self::Stamp>;

    // 元素id × 预算值 → 任务 //

    /// 元素id
    type Key: BagKey;

    /// 预算值
    type Budget: BudgetValueConcrete<E = Self::ShortFloat>;

    /// 任务
    type Task: TaskConcrete<Sentence = Self::Sentence, Key = Self::Key, Budget = Self::Budget>;

    // 词项链 × 任务链 → 概念 → 记忆区 //

    /// 词项链
    type TermLink: TermLinkConcrete<
        Target = Term, // TODO: 后续将「词项」也抽象出一个「特征」来
        Key = Self::Key,
        Budget = Self::Budget,
    >;

    /// 任务链
    type TaskLink: TaskLinkConcrete<Task = Self::Task, Key = Self::Key, Budget = Self::Budget>;

    /// 概念
    type Concept: ConceptConcrete<
        Stamp = Self::Stamp,
        Truth = Self::Truth,
        Sentence = Self::Sentence,
        Key = Self::Key,
        Budget = Self::Budget,
        Task = Self::Task,
        TermLink = Self::TermLink,
        TaskLink = Self::TaskLink,
    >;

    // ! 【2024-05-11 08:56:59】📌↓下面这几个会与「记忆区」冲突，故不约束

    // /// 概念袋
    // type ConceptBag: ConceptBag<Concept = Self::Concept>;

    // /// 词项链袋
    // type TermLinkBag: TermLinkBag<Link = Self::TermLink>;

    // /// 任务链袋
    // type TaskLinkBag: TaskLinkBag<Link = Self::TaskLink>;

    /// 记忆区
    type Memory: MemoryConcrete<
        ShortFloat = Self::ShortFloat,
        Stamp = Self::Stamp,
        Truth = Self::Truth,
        Sentence = Self::Sentence,
        Key = Self::Key,
        Task = Self::Task,
        TermLink = Self::TermLink,
        TaskLink = Self::TaskLink,
        Budget = Self::Budget,
        Concept = Self::Concept,
    >;
}
