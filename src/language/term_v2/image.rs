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

use super::*;

impl Term {
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
    ///
    /// TODO: 【2024-05-11 14:29:23】🏗️后续考虑改为[`Option`]
    ///
    /// # 📄OpenNARS
    ///
    /// get the index of the relation in the component list
    ///
    /// @return the index of relation
    #[doc(alias = "get_relation_index")]
    pub fn get_placeholder_index(&self) -> usize {
        match &&*self.components {
            TermComponents::MultiIndexed(index, _) => *index,
            _ => panic!("尝试获取「非『像』词项」的关系索引"),
        }
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
    pub fn get_relation(&self) -> &Term {
        match &&*self.components {
            TermComponents::MultiIndexed(_, terms) => &terms[0],
            _ => panic!("尝试获取「非『像』词项」的关系词项"),
        }
    }

    /// 📄OpenNARS `getTheOtherComponent` 属性
    /// * 🎯用于获取「像」的「另一词项」
    /// * ⚠️若尝试获取「非『像』词项」的词项，则会panic
    /// * 🆕按NARust「索引=占位符索引」的来：总是在索引`1`处
    ///
    /// # 📄OpenNARS
    ///
    /// Get the other term in the Image
    ///
    /// @return The term related
    pub fn get_the_other_component(&self) -> Option<&Term> {
        /* 📄OpenNARS源码：
        if (components.size() != 2) {
            return null;
        }
        return (relationIndex == 0) ? components.get(1) : components.get(0); */
        match &&*self.components {
            TermComponents::MultiIndexed(_, terms) => match terms.len() {
                2 => Some(&terms[1]),
                _ => None,
            },
            _ => panic!("尝试获取「非『像』词项」的关系词项"),
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::asserts;

    #[test]
    fn instanceof_image() -> AResult {
        asserts! {
            // 像占位符在第一位的「像」会被解析为「乘积」
            term!(r"(/, _, A, B)").identifier() => PRODUCT_OPERATOR
            term!(r"(\, _, A, B)").identifier() => PRODUCT_OPERATOR,
            // 其余正常情况
            Term::new_image_ext(1, vec![term!("S"), term!("A"), term!("B")])?.instanceof_image()
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
            // term!(r"(/, _, A, B)").get_relation_index() => 0 // 会被解析为「乘积」
            // term!(r"(\, _, A, B)").get_relation_index() => 0 // 会被解析为「乘积」
            term!(r"(/, A, _, B)").get_placeholder_index() => 1
            term!(r"(\, A, _, B)").get_placeholder_index() => 1
            term!(r"(/, A, B, _)").get_placeholder_index() => 2
            term!(r"(\, A, B, _)").get_placeholder_index() => 2
        }
        ok!()
    }

    #[test]
    fn get_relation() -> AResult {
        asserts! {
            term!(r"(/, R, _, B)").get_relation() => &term!("R")
            term!(r"(\, R, _, B)").get_relation() => &term!("R")
            term!(r"(/, R, A, _)").get_relation() => &term!("R")
            term!(r"(\, R, A, _)").get_relation() => &term!("R")
        }
        ok!()
    }

    #[test]
    fn get_the_other_component() -> AResult {
        asserts! {
            term!(r"(/, R, _, B)").get_the_other_component() => Some(&term!("B"))
            term!(r"(\, R, _, B)").get_the_other_component() => Some(&term!("B"))
            term!(r"(/, R, A, _)").get_the_other_component() => Some(&term!("A"))
            term!(r"(\, R, A, _)").get_the_other_component() => Some(&term!("A"))
        }
        ok!()
    }
}
