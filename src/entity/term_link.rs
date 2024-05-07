//! 🎯复刻OpenNARS `nars.entity.TermLink`
//! * ✅【2024-05-04 23:10:35】基本完成功能
//! * ✅【2024-05-05 12:13:53】基本完成单元测试

use super::Item;
use crate::{global::RC, io::symbols, language::Term};
use std::fmt::Display;

/// 实现与「词项链类型」相关的结构
/// * 🎯复刻OpenNARS `TermLink.type`与`TermLink.index`
mod link_type {
    /// 指示一个「直接/间接 的」组分 在复合词项中的位置
    /// * 🚩直接表示一个「路径式坐标」
    /// * ⚠️隐式要求合法：路径必须得能找到
    /// * 📄`A` 在 `<(*, A, B) --> C>`中的路径
    ///   * 是(`(*, A, B)`在`<(*, A, B) --> C>`中的路径)/`0`（第一个）
    ///     * `(*, A, B)`在`<(*, A, B) --> C>`中的路径
    ///       * 是`0`（陈述主词）
    ///   * 是`0`/`0`（第一个中的第一个）
    ///   * 因此总索引为`[0, 0]`
    /// * 🚩【2024-05-04 20:35:25】因为「可交换词项」目前表示为「自动排序的词项」，因此不设任何特殊操作
    ///   * ❗亦即：「集合」也是能被索引的
    ///   * 📄`A`在`{A, B}`的位置就是`0`，而非什么「属于/不属于」（或`None`/`Some(具体索引)`）
    pub type ComponentIndex = Vec<usize>;
    /// [`ComponentIndex`]的引用版本
    /// * 🎯【2024-05-04 20:44:24】出于性能考量
    pub type ComponentIndexRef<'a> = &'a [usize];

    /// 词项链引用
    /// * 🚩只表示「连接的『类型』与『属性』」而不表示「连接的『起点』」
    /// * 🎯复刻`TermLink.type`与`TermLink.indexes`字段
    ///   * ✨简并两个字段，而无需额外的假设与判断
    /// * 🚩🆕利用Rust `enum`枚举类型的优势
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum TermLinkRef<'a> {
        /// 与自身的连接
        /// * 📌图式：`C -> C`
        /// * ⚠️仅在任务链中使用
        /// * 🚩【2024-05-04 19:11:04】回避Rust关键词`Self`
        ///
        /// # 📄OpenNARS
        ///
        /// At C, point to C; TaskLink only
        SELF,

        /// 复合词项/组分
        /// * 📌图式：`(&&, A, C)` => `C`
        ///
        /// # 📄OpenNARS
        ///
        /// At (&&, A, C), point to C
        Component,

        /// 复合词项/整体
        /// * 📌图式：`C` => `(&&, A, C)`
        /// * 🚩【2024-05-04 20:30:13】需要一个「位置索引」来获取「组分位置」
        ///
        /// # 📄OpenNARS
        ///
        /// At C, point to (&&, A, C)
        Compound(ComponentIndexRef<'a>),

        /// 陈述/组分
        /// * 📌图式：`<C -- A>` => `C`
        ///
        /// # 📄OpenNARS
        ///
        /// At <C --> A>, point to C
        ComponentStatement,

        /// 陈述/整体
        /// * 📌图式：`C` => `<C -- A>`
        ///
        /// # 📄OpenNARS
        ///
        /// At C, point to <C --> A>
        CompoundStatement(ComponentIndexRef<'a>),

        /// 条件/组分
        /// * 📌图式：`<(&&, C, B) ==> A>` => `C`
        ///
        /// # 📄OpenNARS
        ///
        /// At <(&&, C, B) ==> A>, point to C
        ComponentCondition,

        /// 条件/整体
        /// * 📌图式：`C` => `<(&&, C, B) ==> A>`
        ///
        /// # 📄OpenNARS
        ///
        /// At C, point to <(&&, C, B) ==> A>
        CompoundCondition(ComponentIndexRef<'a>),

        /// 转换
        /// * 📌图式：`C` => `<(*, C, B) --> A>`
        /// * ⚠️仅在任务链中使用
        ///
        /// # 📄OpenNARS
        ///
        /// At C, point to <(*, C, B) --> A>; TaskLink only
        Transform(ComponentIndexRef<'a>),
    }

    impl<'a> TermLinkRef<'a> {
        /// 模拟`TermLink`中的`(type % 2) == 1`
        pub fn is_to_component(&self) -> bool {
            use TermLinkRef::*;
            matches!(self, Component | ComponentStatement | ComponentCondition)
        }

        /// 🆕判断是否有「位置索引」
        /// * 🎯用于在推理中 判断/假定 「是否有位置索引」
        /// * 🚩【2024-05-06 23:02:36】根据英语网站的解释，采用`indexes`而非`indices`
        ///   * 📝后者据称更偏向【数学/统计学】含义
        ///   * 🔗https://www.nasdaq.com/articles/indexes-or-indices-whats-the-deal-2016-05-12
        ///   * 🚩下[`Self::get_indexes`]、[`TermLink::get_indexes`]同
        #[doc(alias = "has_indices")]
        pub fn has_indexes(&self) -> bool {
            use TermLinkRef::*;
            matches!(
                self,
                Compound(..) | CompoundStatement(..) | CompoundCondition(..) | Transform(..)
            )
        }

        /// 🆕尝试获取「位置索引」
        /// * 🚩只对具有「位置索引」的枚举返回[`Some`]
        /// * 🎯用于在推理中获取「是否有位置索引」以便分派规则
        /// * 🚩【2024-05-06 22:56:23】因为可能为空，所以保留`get_`前缀
        /// * 📌此处所返回引用之生命周期，并非`self`的生命周期，而是「其所引用之对象」的生命周期
        ///   * ⚠️`'a`可能比`self`活得更久，参见[`super::TermLink::get_indexes`]的情况
        #[doc(alias = "indexes")]
        #[doc(alias = "indices")]
        #[doc(alias = "get_indices")]
        pub fn get_indexes(&self) -> Option<ComponentIndexRef<'a>> {
            use TermLinkRef::*;
            match *self {
                // 有索引的情况
                Compound(indexes)
                | CompoundStatement(indexes)
                | CompoundCondition(indexes)
                | Transform(indexes) => Some(indexes),
                // 其它情况
                SELF | Component | ComponentStatement | ComponentCondition => None,
            }
        }
    }

    /// [`TermLinkRef`]具备所有权的类型
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TermLinkType {
        /// 与自身的连接
        /// * 📌图式：`C -> C`
        /// * ⚠️仅在任务链中使用
        SELF,

        /// 复合词项/组分
        /// * 📌图式：`(&&, A, C)` => `C`
        ///
        /// # 📄OpenNARS
        ///
        /// At (&&, A, C), point to C
        Component,

        /// 复合词项/整体
        /// * 📌图式：`C` => `(&&, A, C)`
        Compound(ComponentIndex),

        /// 陈述/组分
        /// * 📌图式：`<C -- A>` => `C`
        ComponentStatement,

        /// 陈述/整体
        /// * 📌图式：`C` => `<C -- A>`
        CompoundStatement(ComponentIndex),

        /// 条件/组分
        /// * 📌图式：`<(&&, C, B) ==> A>` => `C`
        ComponentCondition,

        /// 条件/整体
        /// * 📌图式：`C` => `<(&&, C, B) ==> A>`
        CompoundCondition(ComponentIndex),

        /// 转换
        /// * 📌图式：`C` => `<(*, C, B) --> A>`
        /// * ⚠️仅在任务链中使用
        Transform(ComponentIndex),
    }

    impl TermLinkType {
        /// 转换为引用类型
        /// * 🎯将「具所有权类型」转换为「类引用类型」
        pub fn to_ref(&self) -> TermLinkRef {
            use TermLinkType::*;
            match self {
                SELF => TermLinkRef::SELF,
                Component => TermLinkRef::Component,
                Compound(vec) => TermLinkRef::Compound(vec),
                ComponentStatement => TermLinkRef::ComponentStatement,
                CompoundStatement(vec) => TermLinkRef::CompoundStatement(vec),
                ComponentCondition => TermLinkRef::ComponentCondition,
                CompoundCondition(vec) => TermLinkRef::CompoundCondition(vec),
                Transform(vec) => TermLinkRef::Transform(vec),
            }
        }
    }

    /// 从引用类型中转换
    impl From<TermLinkRef<'_>> for TermLinkType {
        fn from(value: TermLinkRef<'_>) -> Self {
            use TermLinkRef::*;
            match value {
                SELF => Self::SELF,
                Component => Self::Component,
                Compound(vec) => Self::Compound(vec.to_owned()),
                ComponentStatement => Self::ComponentStatement,
                CompoundStatement(vec) => Self::CompoundStatement(vec.to_owned()),
                ComponentCondition => Self::ComponentCondition,
                CompoundCondition(vec) => Self::CompoundCondition(vec.to_owned()),
                Transform(vec) => Self::Transform(vec.to_owned()),
            }
        }
    }

    /// 与[`TermLinkRef`]作比较
    /// * 🎯允许更高性能地直接与[`TermLinkRef`]判等，而无需创建新值
    impl PartialEq<TermLinkRef<'_>> for TermLinkType {
        fn eq(&self, other: &TermLinkRef) -> bool {
            // 简化以下匹配代码
            use TermLinkType::*;
            type Ref<'a> = TermLinkRef<'a>;
            // 开始匹配
            match (self, other) {
                // 类型相同，无附加参数
                (SELF, Ref::SELF)
                | (Component, Ref::Component)
                | (ComponentStatement, Ref::ComponentStatement)
                | (ComponentCondition, Ref::ComponentCondition) => true,
                // 类型相同，附加参数相同
                (Compound(vec), Ref::Compound(vec2))
                | (CompoundStatement(vec), Ref::CompoundStatement(vec2))
                | (CompoundCondition(vec), Ref::CompoundCondition(vec2))
                | (Transform(vec), Ref::Transform(vec2)) => vec == vec2,
                // 类型不同
                _ => false,
            }
        }
    }
}
pub use link_type::*;

/// 模拟OpenNARS `nars.entity.TermLink`
/// * 🚩首先是一个「Item」
/// * ❓【2024-05-06 00:08:34】目前「词项链」和「[『词项』](Term)链」并没分开来，似乎是个不好的习惯
///   * ❓到底「任务链」应不应该继承「词项链」
///   * 💭或许这俩应该分开，至少现在这个[`TermLink`]应该改成`TargetLink`或者别的什么抽象特征
///   * 📌然后[`TermLink`]就是`TargetLink<Target = Term>`这样
///
/// TODO: 🏗️【2024-05-06 00:10:28】↑后续再行动，优化复用情况
///
/// # 📄OpenNARS
///
/// A link between a compound term and a component term
///
/// A TermLink links the current Term to a target Term, which is
/// either a component of, or compound made from, the current term.
///
/// Neither of the two terms contain variable shared with other terms.
///
/// The index value(s) indicates the location of the component in the compound.
///
/// This class is mainly used in inference.RuleTable to dispatch premises to
/// inference rules
pub trait TermLink: Item {
    /// 连接所基于的「目标」
    /// * 📌可以是[词项](Term)，亦可为[任务](super::Task)
    /// * ❓目前似乎需要为「词项」实现一个特征，然后将约束限定在「词项」上
    ///   * ❗这样才能至少使用「词项」的功能
    ///   * 📄如「通过[`Display`]生成[『元素id』](crate::storage::BagKey)」
    type Target: Display;

    /// 🆕根据自身生成[`Item::key`]
    /// * 🎯可复用、无副作用的「字符串生成」逻辑
    /// * 🔗OpenNARS源码参见[`TermLink::_set_key`]
    /// * 🚩【2024-05-04 23:20:50】现在升级为静态方法，无需`self`
    ///   * 🎯为了「在构造之前生成key」
    /// * 🚩现不再提供默认的[`String`]实现，以便完全和字符串[`String`]解耦
    fn _generate_key(target: &Self::Target, type_ref: TermLinkRef) -> Self::Key;

    /// 模拟`TermLink.setKey`
    /// * 🚩将自身信息转换为用于「唯一标识」的「袋元素id」
    ///
    /// # 📄OpenNARS
    ///
    /// Set the key of the link
    fn _set_key(&mut self) {
        /* 📄OpenNARS源码：
        String at1, at2;
        if ((type % 2) == 1) { // to component
            at1 = Symbols.TO_COMPONENT_1;
            at2 = Symbols.TO_COMPONENT_2;
        } else { // to compound
            at1 = Symbols.TO_COMPOUND_1;
            at2 = Symbols.TO_COMPOUND_2;
        }
        String in = "T" + type;
        if (index != null) {
            for (int i = 0; i < index.length; i++) {
                in += "-" + (index[i] + 1);
            }
        }
        key = at1 + in + at2;
        if (target != null) {
            key += target;
        } */
        // 🆕直接生成并赋值
        *self.__key_mut() = Self::_generate_key(self.target(), self.type_ref());
    }

    /// 🆕模拟[`Item::key`]的可变版本
    /// * 🎯在模拟`TermLink.setKey`时要用于赋值
    fn __key_mut(&mut self) -> &mut Self::Key;

    /// 模拟`TermLink.target`
    /// * 📝链接所归属的词项
    /// * 📝链接「At」的起点
    /// * 🚩对外只读
    /// * 🚩🆕对于「任务链」，OpenNARS中会返回`null`，此处不采取这种做法
    ///   * 🚩【2024-05-04 23:04:54】目前做法：直接取[`TaskLink::target_task`]中包含的[`Task::term`]属性
    ///   * 📌这样能保证「总是有值」，可以在「生成key」中省去一次判空
    ///
    /// # 📄OpenNARS
    ///
    /// - The linked Term
    /// - Get the target of the link
    ///
    /// @return The Term pointed by the link
    fn target(&self) -> &Self::Target;

    /// 模拟`TermLink.type`
    /// * 🚩【2024-05-04 22:42:10】回避Rust关键字`type`
    /// * 🚩对外只读，对子类开放
    #[doc(alias = "link_type")]
    #[doc(alias = "link_type_ref")]
    fn type_ref(&self) -> TermLinkRef;

    /// 模拟`TermLink.getIndices`
    /// * 🚩通过[`TermLink::type_ref`]直接获取
    /// * ⚠️可能为空
    #[inline(always)]
    #[doc(alias = "get_indices")]
    #[doc(alias = "indices")]
    fn get_indexes(&self) -> Option<ComponentIndexRef> {
        self.type_ref().get_indexes()
    }

    /// 模拟`TermLink.getIndex`
    /// * 🚩通过[`TermLink::type_ref`]直接获取
    /// * ⚠️可能为空
    #[inline(always)]
    #[doc(alias = "index")]
    #[doc(alias = "get")]
    fn get_index(&self, index: usize) -> Option<usize> {
        self.type_ref().get_indexes().map(|indexes| indexes[index])
    }

    // * 📝OpenNARS始终将这俩方法用在「规则表的分派」中，并且总是会对「词项链类型」做分派
}

/// 具体的「词项链」类型
/// * 🚩将原先的「词项链」变成真正的「[词项](Term)链」
/// * 🚩在原有的「词项链」基础上增加
pub trait TermLinkConcrete: TermLink<Target = Term> + Sized {
    /// 模拟 `new TermLink(Term t, short p, int... indices)`
    /// * 🚩需要「词项」「链接」「预算值」
    ///
    /// # 📄OpenNARS
    ///
    /// Constructor for TermLink template
    ///
    /// called in CompoundTerm.prepareComponentLinks only
    ///
    /// @param t       Target Term
    /// @param p       Link type
    /// @param indices Component indices in compound, may be 1 to 4
    fn new(
        budget: Self::Budget,
        target: impl Into<RC<Self::Target>>,
        type_ref: TermLinkType,
    ) -> Self;

    // TODO: 复现其它构造函数
    // TODO: 模拟 `new TermLink(String s, BudgetValue v)`
    // TODO: 模拟 `new TermLink(Term t, TermLink template, BudgetValue v)`
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::entity::BudgetValueConcrete;

    /// 词项链 初代实现
    /// * 🚩目前不限制其中「预算值」的类型
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TermLinkV1<B: BudgetValueConcrete> {
        key: String,
        budget: B,
        target: RC<Term>,
        type_ref: TermLinkType,
    }

    impl<B: BudgetValueConcrete> TermLinkConcrete for TermLinkV1<B> {
        /// 构造函数
        /// * 📌包含「预算」「目标词项」「类型」
        /// * 🚩其key是自行计算的
        fn new(budget: B, target: impl Into<RC<Term>>, type_ref: TermLinkType) -> Self {
            let target = target.into();
            Self {
                key: Self::_generate_key(&target, type_ref.to_ref()),
                budget,
                target,
                type_ref,
            }
        }
    }

    impl<B: BudgetValueConcrete> Item for TermLinkV1<B> {
        type Key = String;
        type Budget = B;

        fn key(&self) -> &String {
            &self.key
        }

        fn budget(&self) -> &B {
            &self.budget
        }

        fn budget_mut(&mut self) -> &mut Self::Budget {
            &mut self.budget
        }
    }

    impl<B: BudgetValueConcrete> TermLink for TermLinkV1<B> {
        type Target = Term;

        fn target(&self) -> &Self::Target {
            &self.target
        }

        fn type_ref(&self) -> TermLinkRef {
            self.type_ref.to_ref()
        }

        fn __key_mut(&mut self) -> &mut String {
            &mut self.key
        }

        fn _generate_key(target: &Self::Target, type_ref: TermLinkRef) -> Self::Key {
            use symbols::*;
            let (at1, at2) = match type_ref.is_to_component() {
                true => (TO_COMPONENT_1, TO_COMPONENT_2),
                false => (TO_COMPOUND_1, TO_COMPOUND_2),
            };
            // 🆕直接格式化 | 🎯只要保证「能展示链接类型和链接索引」即可
            format!("{at1}T-{type_ref:?}{at2}{target}") // ! 注意：at2里边已经包含空格
        }
    }
}
pub use impl_v1::*;

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity::{BudgetV1, BudgetValueConcrete},
        global::tests::AResult,
        ok, test_term,
    };
    use std::str::FromStr;

    /// 用于测试的预算值类型
    type Budget = BudgetV1;
    /// 用于测试的词项链类型
    type TL = TermLinkV1<Budget>;

    /// 构造 & 展示
    /// * 🎯构造 [`TL::new`]
    /// * 🎯展示 [`TL::key`]
    #[test]
    fn new() -> AResult {
        let tl = TL::new(
            Budget::from_floats(0.5, 0.5, 0.5),
            Term::new_word("term"),
            TermLinkType::SELF,
        );
        let tl2 = TL::new(
            Budget::from_floats(0.1, 0.5, 1.0),
            test_term!("<(*, {A, B}) --> C>"),
            // ? `<(*, {A, B}) --> C>` => A
            TermLinkType::CompoundStatement(vec![0, 0]),
        );
        let show = |tl: &TL| println!("tl = {:?}; key = {:?}", dbg!(tl), tl.key());
        show(&tl);
        show(&tl2);

        ok!()
    }

    // * ✅测试/_generate_key 已在[`new`]中测试

    /// 测试/_set_key
    #[test]
    fn _set_key() -> AResult {
        // 新建词项链
        let mut tl = TL::new(
            Budget::from_floats(0.5, 0.5, 0.5),
            Term::new_word("term"),
            TermLinkType::SELF,
        );
        // 默认不应该为空
        assert!(!tl.key().is_empty());
        // ! 强行修改key
        *tl.__key_mut() = "".into();
        // 改了之后就被清空了
        assert!(tl.key().is_empty());
        // 重新设置
        tl._set_key();
        // 设置之后不该为空
        assert!(!tl.key().is_empty());
        // 完成
        ok!()
    }

    // * ✅测试/__key_mut已在[`_set_key`]中测试

    /// 测试/target
    #[test]
    fn target() -> AResult {
        // 新建词项
        let term = Term::from_str("<{(*, A), B, C} ==> <D --> E>>")?;
        // 装入词项链
        let tl = TL::new(Budget::default(), term.clone(), TermLinkType::SELF);
        // 应该一致
        assert_eq!(term, *tl.target());
        // 完成
        ok!()
    }

    /// 测试/type_ref
    /// * 🎯[`TermLink::type_ref`]
    /// * 🎯[`TermLinkType::from`]
    /// * 🎯[`TermLinkType::to_ref`]
    #[test]
    fn type_ref() -> AResult {
        // 新建词项链类型
        let link = TermLinkType::CompoundCondition(vec![
            'A' as usize,
            'R' as usize,
            'C' as usize,
            'J' as usize,
            '1' as usize,
            '3' as usize,
            '7' as usize,
            '4' as usize,
            '4' as usize,
            '2' as usize,
        ]);
        // 装入词项链
        let tl = TL::new(Budget::default(), Term::from_str("term")?, link.clone());
        // 应该一致
        assert_eq!(link, tl.type_ref());
        // 转换后应该一致
        assert_eq!(link.to_ref(), tl.type_ref());
        assert_eq!(link, TermLinkType::from(tl.type_ref()));
        // 完成
        ok!()
    }
}
