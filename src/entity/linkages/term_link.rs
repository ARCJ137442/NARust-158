//! 词项链

use super::{TLink, TLinkType, TLinkage, TermLinkTemplate};
use crate::{
    entity::{BudgetValue, Item, ShortFloat, Token},
    inference::Budget,
    language::Term,
    util::ToDisplayAndBrief,
};
use nar_dev_utils::join;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// 词项链
///
/// # 📄OpenNARS
///
/// A link between a compound term and a component term
///
/// A TermLink links the current Term to a target Term, which is either a component of, or compound made from, the current term.///
///
/// Neither of the two terms contain variable shared with other terms.///
///
/// The index value(s) indicates the location of the component in the compound.///
///
/// This class is mainly used in inference.RuleTable to dispatch premises to inference rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TermLink {
    /// 🆕纯粹的T链接类型
    inner: TLinkage<Term>,
    /// 🆕Item令牌
    token: Token,
}

impl TermLink {
    fn new(
        target: Term,
        budget: BudgetValue,
        link_type: TLinkType,
        indexes: impl Into<Box<[usize]>>,
    ) -> Self {
        let indexes = indexes.into();
        let key = Self::generate_key_for_term_link(&target, link_type, &indexes);
        let inner = TLinkage::new_direct(target, link_type, indexes);
        Self {
            token: Token::new(key, budget),
            inner,
        }
    }

    pub fn from_template(target: Term, template: &TermLinkTemplate, budget: BudgetValue) -> Self {
        // * 🚩生成类型与索引
        let link_type = Self::generate_type_from_template(&target, template);
        let indexes = template.indexes().to_vec().into_boxed_slice();
        // * 🚩构造
        Self::new(target, budget, link_type, indexes)
    }

    /// 不同于默认方法，但要调用默认方法
    /// * 🎯从「目标」、已生成的「类型」「索引」生成「键」
    fn generate_key_for_term_link(
        target: &Term,
        link_type: TLinkType,
        indexes: &[usize],
    ) -> String {
        // * 🚩标准T链接子串 + 词项的字符串形式
        Self::generate_key_base(link_type, indexes) + &target.to_string()
    }

    fn generate_type_from_template(target: &Term, template: &TermLinkTemplate) -> TLinkType {
        let template_type = template.link_type();
        // * 🚩断言此时「链接模板」的链接类型：必定是「从元素链接到整体」
        debug_assert!(
            template_type.is_to_compound(),
            "模板必定是「从元素链接到整体」"
        );
        // * 🚩开始计算类型
        match template.will_from_self_to() == target {
            // * 🚩自「元素→整体」来（复合词项的「模板链接」指向自身）
            // * 🚩到「整体→元素」去
            // * 📄【2024-06-04 20:35:22】
            // * Concept@48 "<{tim} --> (/,livingIn,_,{graz})>" ~> target="{tim}"
            // * + template: willFromSelfTo="{tim}"
            // * 📄【2024-06-04 20:35:32】
            // * Concept@52 "<{tim} --> (/,livingIn,_,{graz})>" ~> target="tim"
            // * + template: willFromSelfTo="tim"
            true => template_type.try_point_to_component(),
            false => template_type,
        }
    }
}

// 委托[`Token`]实现
impl Budget for TermLink {
    fn priority(&self) -> ShortFloat {
        self.token.priority()
    }

    fn __priority_mut(&mut self) -> &mut ShortFloat {
        self.token.__priority_mut()
    }

    fn durability(&self) -> ShortFloat {
        self.token.durability()
    }

    fn __durability_mut(&mut self) -> &mut ShortFloat {
        self.token.__durability_mut()
    }

    fn quality(&self) -> ShortFloat {
        self.token.quality()
    }

    fn __quality_mut(&mut self) -> &mut ShortFloat {
        self.token.__quality_mut()
    }
}

// 委托[`Token`]实现
impl Item for TermLink {
    type Key = String;
    fn key(&self) -> &String {
        self.token.key()
    }
}

// 委托[`TLinkage`]实现
impl TLink<Term> for TermLink {
    fn target<'r, 's: 'r>(&'s self) -> impl Deref<Target = Term> + 'r {
        self.inner.target()
    }

    fn target_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = Term> + 'r {
        self.inner.target_mut()
    }

    fn link_type(&self) -> super::TLinkType {
        self.inner.link_type()
    }

    fn indexes(&self) -> &[usize] {
        self.inner.indexes()
    }
}

impl ToDisplayAndBrief for TermLink {
    fn to_display(&self) -> String {
        join! {
            => self.token.budget_to_display()
            => " "
            => self.key()
        }
    }

    fn to_display_brief(&self) -> String {
        join! {
            => self.token.budget_to_display_brief()
            => " "
            => self.key()
        }
    }
}
