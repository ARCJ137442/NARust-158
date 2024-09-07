//! 📄OpenNARS `nars.language.ImageXXt`
//! * 🎯复刻OpenNARS中有关「外延像/内涵像」的通用函数
//! * 📌NAL底层的「像」逻辑，对应`ImageExt`与`ImageInt`
//! * ⚠️不包括与记忆区有关的`make`系列方法
//!
//! # 🆕差异点
//! ! ⚠️因原先语法上实现的差异，此处对「像占位符位置」与「关系词项位置」的表述，和OpenNARS不一样
//! * 📝NAL-4中，「像」作为一种「关系与参数中挖了空的词项」，
//!   * 将**第一位**固定为「关系词项」的位置
//! * 📌范围：1~总词项数
//!   * ⚠️第一位为「关系词项」预留——若有占位符，则与「乘积」无异
//! * 📌核心差异：自Narsese.rs以来，NARust更强调「占位符的词法位置」而非「关系词项之后的位置」
//!
//! ## 例
//!
//! 对`(*,A,B) --> P`
//! * 在第1位
//!   * 📄OpenNARS: ⇔ `A --> (/,P,_,B)` ⇒ `(/,P,B)_0`
//!   * 📄NARust:   ⇔ `A --> (/,P,_,B)` ⇒ `(/,P,B)_1`
//!     * 📌`1`而非`0`的依据：占位符在「像」中的位置为`1`
//! * 在第2位
//!   * 📄OpenNARS: ⇔ `B --> (/,P,A,_)` ⇒ `(/,A,P)_1`
//!   * 📄NARust:   ⇔ `B --> (/,P,A,_)` ⇒ `(/,P,A)_2`
//!     * 📌`2`而非`1`的依据：占位符在「像」中的位置为`2`
//! * 在第0位（扩展）
//!   * 📄OpenNARS: 【不支持】（会自动转换到「第一位」去）
//!   * 📄NARust:   ⇔ `P --> (/,_,A,B)` ⇒ `(/,A,B)_0`
//!     * 📌`0`的依据：占位符在「像」中的位置为`0`
//!     * ❓PyNARS却又支持`(/,_,A)`，但又把`<P --> (/,_,A,B)>.`推导成`<(*, A, B)-->_>.`
//!
//! # 方法列表
//! 🕒最后更新：【2024-04-24 20:15:43】
//!
//! * `ImageExt` / `ImageInt`
//!   * `getRelationIndex`
//!   * `getRelation`
//!   * `getTheOtherComponent`
//!
//! # 📄OpenNARS
//!
//! ## 外延像
//! An extension image.
//!
//! `B --> (/,P,A,_)` iff `(*,A,B) --> P`
//!
//! Internally, it is actually `(/,A,P)_1`, with an index.
//!
//! ## 内涵像
//! An intension image.
//!
//! `(\,P,A,_) --> B` iff `P --> (*,A,B)`
//!
//! Internally, it is actually `(\,A,P)_1`, with an index.

use crate::language::*;
use nar_dev_utils::matches_or;

impl<'a> CompoundTermRef<'a> {
    // * ✅现在「判别函数」统一迁移至[`super::compound`]

    /// 📄OpenNARS `getRelationIndex` 属性
    /// * 🎯用于获取「像」的关系索引
    /// * 🆕⚠️现在是获取「占位符位置」
    ///   * 📝原先OpenNARS是将「关系词项」放在占位符处的，现在是根据《NAL》原意，将「关系词项」统一放在「第一个词项」处
    ///   * 📌所以后续所有的「索引」都变成了「占位符位置」
    ///   * 💭【2024-05-11 14:40:15】后续可能会在这点上有隐患——随后要注意这种差别
    ///
    /// # Panics
    ///
    /// ! ⚠️仅限于「像」的`TermComponents::MultiIndexed`词项
    /// * 若尝试获取「非『像』词项」的关系索引，则会panic
    /// * 🚩【2024-06-12 22:53:09】本来就不应该对「非像词项」调用该函数——严格跟「像」类型绑定
    ///
    /// # 📄OpenNARS
    ///
    /// get the index of the relation in the component list
    ///
    /// @return the index of relation
    #[doc(alias = "get_relation_index")]
    pub fn get_placeholder_index(self) -> usize {
        self.components
            .iter()
            .position(Term::is_placeholder)
            .expect("尝试获取「非『像』词项」的关系索引")
    }

    /// 📄OpenNARS `getRelation` 属性
    /// * 🎯用于获取「像」的「关系词项」
    /// * ⚠️若尝试获取「非『像』词项」的关系词项，则会panic
    /// * 🆕按NARust「索引=占位符索引」的来：总是在索引`0`处
    ///
    /// # 📄OpenNARS
    ///
    /// Get the relation term in the Image
    ///
    /// @return The term representing a relation
    pub fn get_relation(self) -> &'a Term {
        &self.components[0]
    }

    /// 📄OpenNARS `getTheOtherComponent` 属性
    /// * 🎯用于获取「像」的「另一词项」
    /// * ⚠️若尝试获取「非『像』词项」的词项，则会panic
    ///
    /// # 📄OpenNARS
    ///
    /// Get the other term in the Image
    ///
    /// @return The term related
    pub fn get_the_other_component(self) -> Option<&'a Term> {
        /* 📄OpenNARS源码：
        if (components.size() != 2) {
            return null;
        }
        return (relationIndex == 0) ? components.get(1) : components.get(0); */
        matches_or! {
            ?self.components,
            // ! 🚩【2024-06-13 23:52:06】现在「占位符」算作一个词项了
            // * 📄[R, _, A]
            [_, term1, term2] => match term1.is_placeholder() {
                true => term2,
                false => term1,
            }
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::*;
    use crate::test_compound as compound;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::asserts;

    #[test]
    fn instanceof_image() -> AResult {
        /// 📌工具方法：直接调用`make_image_xxt_vec`构造词项
        fn make_image(
            argument: impl IntoIterator<Item = &'static str>,
            make_vec: impl Fn(Vec<Term>) -> Option<Term>,
        ) -> AResult<Term> {
            let argument = argument
                .into_iter()
                .map(|t| t.parse::<Term>().expect("内部词项解析失败"))
                .collect::<Vec<_>>();
            make_vec(argument).ok_or(anyhow::anyhow!("词项解析失败"))
        }
        fn make_ext(argument: impl IntoIterator<Item = &'static str>) -> AResult<Term> {
            make_image(argument, Term::make_image_ext_vec)
        }
        fn make_int(argument: impl IntoIterator<Item = &'static str>) -> AResult<Term> {
            make_image(argument, Term::make_image_int_vec)
        }
        asserts! {
            // 像占位符在第一位的「像」会被解析为「乘积」
            term!(r"(/, _, A, B)").identifier() => PRODUCT_OPERATOR,
            term!(r"(\, _, A, B)").identifier() => PRODUCT_OPERATOR,
            // 其余正常情况
            make_ext(["S", "_", "A", "B"])?.instanceof_image()
            make_int(["S", "_", "A", "B"])?.instanceof_image()
            make_ext(["S", "A", "_", "B"])?.instanceof_image()
            make_int(["S", "A", "_", "B"])?.instanceof_image()
            make_ext(["S", "A", "B", "_"])?.instanceof_image()
            make_int(["S", "A", "B", "_"])?.instanceof_image()
            term!(r"(/, A, _, B)").instanceof_image()
            term!(r"(\, A, _, B)").instanceof_image()
            term!(r"(/, A, B, _)").instanceof_image()
            term!(r"(\, A, B, _)").instanceof_image()
        }
        ok!()
    }

    #[test]
    fn get_relation_index() -> AResult {
        asserts! {
            // compound!(r"(/, _, A, B)").get_relation_index() => 0 // 会被解析为「乘积」
            // compound!(r"(\, _, A, B)").get_relation_index() => 0 // 会被解析为「乘积」
            compound!(r"(/, A, _, B)").get_placeholder_index() => 1
            compound!(r"(\, A, _, B)").get_placeholder_index() => 1
            compound!(r"(/, A, B, _)").get_placeholder_index() => 2
            compound!(r"(\, A, B, _)").get_placeholder_index() => 2
        }
        ok!()
    }

    #[test]
    fn get_relation() -> AResult {
        asserts! {
            compound!(r"(/, R, _, B)").get_relation() => &term!("R")
            compound!(r"(\, R, _, B)").get_relation() => &term!("R")
            compound!(r"(/, R, A, _)").get_relation() => &term!("R")
            compound!(r"(\, R, A, _)").get_relation() => &term!("R")
        }
        ok!()
    }

    #[test]
    fn get_the_other_component() -> AResult {
        asserts! {
            compound!(r"(/, R, _, B)").get_the_other_component() => Some(&term!("B"))
            compound!(r"(\, R, _, B)").get_the_other_component() => Some(&term!("B"))
            compound!(r"(/, R, A, _)").get_the_other_component() => Some(&term!("A"))
            compound!(r"(\, R, A, _)").get_the_other_component() => Some(&term!("A"))
        }
        ok!()
    }
}
