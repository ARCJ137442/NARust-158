//! 🎯复刻OpenNARS `nars.entity.TermLink`
//! * ✅【2024-05-04 23:10:35】基本完成功能
//! * ✅【2024-05-05 12:13:53】基本完成单元测试

use super::Item;
use crate::{io::symbols, language::Term};

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

    impl TermLinkRef<'_> {
        /// 模拟`TermLink`中的`(type % 2) == 1`
        pub fn is_to_component(&self) -> bool {
            use TermLinkRef::*;
            matches!(self, Component | ComponentStatement | ComponentCondition)
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
}
pub use link_type::*;

/// 模拟OpenNARS `nars.entity.TermLink`
/// * 🚩首先是一个「Item」
pub trait TermLink: Item + Sized {
    // ! 🚩【2024-05-04 20:49:09】暂不模拟构造函数
    // /// 模拟 `TermLink`构造函数
    // /// * 🚩需要「词项」「链接」「预算值」
    // fn new(t: &Term, link: ComponentIndexRef) -> Self;

    /// 🆕根据自身生成[`Item::key`]
    /// * 🎯可复用、无副作用的「字符串生成」逻辑
    /// * 🔗OpenNARS源码参见[`TermLink::_set_key`]
    /// * 🚩【2024-05-04 23:20:50】现在升级为静态方法，无需`self`
    ///   * 🎯为了「在构造之前生成key」
    fn _generate_key(target: &Term, type_ref: TermLinkRef) -> String {
        use symbols::*;
        let (at1, at2) = match type_ref.is_to_component() {
            true => (TO_COMPONENT_1, TO_COMPONENT_2),
            false => (TO_COMPOUND_1, TO_COMPOUND_2),
        };
        // 🆕直接格式化 | 🎯只要保证「能展示链接类型和链接索引」即可
        format!("{at1}T-{type_ref:?}{at2}{target}") // ! 注意：at2里边已经包含空格
    }

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
    fn __key_mut(&mut self) -> &mut String;

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
    fn target(&self) -> &Term;

    /// 模拟`TermLink.type`
    /// * 🚩【2024-05-04 22:42:10】回避Rust关键字`type`
    /// * 🚩对外只读，对子类开放
    fn type_ref(&self) -> TermLinkRef;

    // * ✅无需模拟`TermLink.getIndices`——其已包含在[`TermLink::type_ref`]中
    // * ✅无需模拟`TermLink.getIndex`——其已包含在[`TermLink::type_ref`]中
    // * 📝OpenNARS始终将这俩方法用在「规则表的分派」中，并且总是会对「词项链类型」做分派
}

/// 初代实现
mod impl_v1 {
    use super::*;
    use crate::entity::BudgetValueConcrete;

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

    /// 词项链 初代实现
    /// * 🚩目前不限制其中「预算值」的类型
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TermLinkV1<B: BudgetValueConcrete> {
        key: String,
        budget: B,
        target: Term,
        type_ref: TermLinkType,
    }

    impl<B: BudgetValueConcrete> TermLinkV1<B> {
        /// 构造函数
        /// * 📌包含「预算」「目标词项」「类型」
        /// * 🚩其key是自行计算的
        pub fn new(budget: B, target: Term, type_ref: TermLinkType) -> Self {
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
        fn target(&self) -> &Term {
            &self.target
        }

        fn type_ref(&self) -> TermLinkRef {
            self.type_ref.to_ref()
        }

        fn __key_mut(&mut self) -> &mut String {
            &mut self.key
        }
    }
}
pub use impl_v1::*;

/// 单元测试
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        entity::{BudgetV1, BudgetValueConcrete},
        global::tests::AResult,
        ok, test_term,
    };

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
            Budget::from_float(0.5, 0.5, 0.5),
            Term::new_word("term"),
            TermLinkType::SELF,
        );
        let tl2 = TL::new(
            Budget::from_float(0.1, 0.5, 1.0),
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
            Budget::from_float(0.5, 0.5, 0.5),
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
            1,
            3,
            7,
            4,
            4,
            2,
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
