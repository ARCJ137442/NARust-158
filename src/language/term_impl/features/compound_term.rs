//! 📄OpenNARS `nars.language.CompoundTerm`
//! * ⚠️不包含与NAL-6有关的「变量」逻辑
//!   * 📄`isConstant`、`renameVariables`
//! * ⚠️不包含与「记忆区」有关的方法
//!   * 📄`addComponents`、`reduceComponents`
//! * ✅【2024-06-14 13:41:30】初步完成对其内方法的更新
//! * ✅【2024-06-14 14:43:30】初步完成单元测试
//!
//! # 方法列表
//! 🕒最后更新：【2024-06-14 10:29:57】
//!
//! * `isCommutative`
//! * `size`
//! * `componentAt`
//! * `getComponents`
//! * `cloneComponents`
//! * `containComponent`
//! * `containTerm`
//! * `containAllComponents`
//! * `setTermWhenDealingVariables`
//! * `updateAfterRenameVariables`
//! * `updateNameAfterRenameVariables`
//! * `reorderComponents`
//!
//! # 📄OpenNARS
//!
//! A CompoundTerm is a Term with internal (syntactic) structure
//!
//! A CompoundTerm consists of a term operator with one or more component Terms.
//!
//! This abstract class contains default methods for all CompoundTerms.

use crate::io::symbols::*;
use crate::language::*;
use nar_dev_utils::matches_or;
use narsese::api::{GetCapacity, TermCapacity};
use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

/// 对词项数组的外加方法
/// * 🎯复现OpenNARS中ArrayList的remove, removeAll等方法
pub(in crate::language) mod vec_utils {
    use crate::language::Term;

    /// 从[`Vec`]中移除一个词项
    pub fn remove(vec: &mut Vec<Term>, term: &Term) -> bool {
        /* 📄Java ArrayList
        final Object[] es = elementData;
        final int size = this.size;
        int i = 0;
        found: {
            if (o == null) {
                for (; i < size; i++)
                    if (es[i] == null)
                        break found;
            } else {
                for (; i < size; i++)
                    if (o.equals(es[i]))
                        break found;
            }
            return false;
        }
        fastRemove(es, i);
        return true; */
        let position = vec.iter().position(|t| t == term);
        match position {
            Some(i) => {
                vec.remove(i);
                true
            }
            None => false,
        }
    }

    /// 在[`Vec`]中移除多个词项
    pub fn remove_all(vec: &mut Vec<Term>, terms: &[Term]) -> bool {
        // * 🚩暂且直接遍历做删除
        // vec.retain(|t| !terms.contains(t)); // ! 📌【2024-06-16 11:59:47】不使用：可能对一个term in terms会删掉多个词项
        let mut removed = false;
        for term in terms {
            // * 🚩始终运行，不使用惰性的any
            if remove(vec, term) {
                removed = true;
            }
        }
        removed
    }

    /// 词项数组取交集
    /// * 📌根据[`==`](Eq::eq)
    pub fn retain_all(vec: &mut Vec<Term>, terms: &[Term]) {
        vec.retain(|t| terms.contains(t));
    }
}

// 词项与「复合词项」（内部元素）无关的特性
impl Term {
    /// 🆕用于判断是否为「纯复合词项」
    /// * ⚠️**不**包括陈述
    pub fn instanceof_compound_pure(&self) -> bool {
        matches!(
            self.identifier.as_str(),
            SET_EXT_OPERATOR
                | SET_INT_OPERATOR
                | INTERSECTION_EXT_OPERATOR
                | INTERSECTION_INT_OPERATOR
                | DIFFERENCE_EXT_OPERATOR
                | DIFFERENCE_INT_OPERATOR
                | PRODUCT_OPERATOR
                | IMAGE_EXT_OPERATOR
                | IMAGE_INT_OPERATOR
                | CONJUNCTION_OPERATOR
                | DISJUNCTION_OPERATOR
                | NEGATION_OPERATOR
                | SEQUENTIAL_CONJUNCTION_OPERATOR
                | PARALLEL_CONJUNCTION_OPERATOR
        )
    }

    /// 🆕用于判断词项是否为「指定类型的复合词项」，并尝试返回「复合词项」的引用信息
    /// * 📌包括陈述
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回不可变引用
    #[must_use]
    pub fn as_compound_type(&self, compound_class: impl AsRef<str>) -> Option<CompoundTermRef> {
        matches_or! {
            ?self.as_compound(),
            Some(compound)
                // * 🚩标识符相等
                if compound_class.as_ref() == self.identifier()
                // * 🚩内部（类型相等）的复合词项
                => compound
        }
    }

    /// 🆕用于判断词项是否为「指定类型的复合词项」
    /// * 📌包括陈述
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_compound_components(self) -> Option<Box<[Term]>> {
        matches_or! {
            ?self,
            // * 🚩匹配到如下结构⇒返回Some，否则返回None
            Term {
                // * 🚩内容为「复合词项」
                components: TermComponents::Compound(terms),
                ..
            }
            // * 🚩返回内容
            => terms
        }
    }

    /// 🆕用于判断词项是否为「指定类型的复合词项」
    /// * 📌包括陈述
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_compound_type_components(
        self,
        compound_class: impl AsRef<str>,
    ) -> Option<Box<[Term]>> {
        matches_or! {
            ?self,
            // * 🚩匹配到如下结构⇒返回Some，否则返回None
            Term {
                identifier,
                // * 🚩内容为「复合词项」
                components: TermComponents::Compound(terms),
                ..
            }
            // * 🚩标识符相等
            if identifier.as_str() == compound_class.as_ref()
            // * 🚩返回内容
            => terms
        }
    }

    /// 🆕用于判断是否为「复合词项」
    /// * ⚠️包括陈述
    /// * 📄OpenNARS `instanceof CompoundTerm` 逻辑
    #[inline(always)]
    pub fn instanceof_compound(&self) -> bool {
        self.instanceof_compound_pure() || self.instanceof_statement()
    }

    /// 🆕用于判断是否为「外延集」
    /// * 📄OpenNARS`instanceof SetExt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_set_ext(&self) -> bool {
        self.identifier == SET_EXT_OPERATOR
    }

    /// 🆕用于判断是否为「内涵集」
    /// * 📄OpenNARS`instanceof SetInt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_set_int(&self) -> bool {
        self.identifier == SET_INT_OPERATOR
    }

    /// 🆕用于判断是否为「词项集」
    /// * 📄OpenNARS`instanceof SetExt || instanceof SetInt`逻辑
    #[inline(always)]
    pub fn instanceof_set(&self) -> bool {
        self.instanceof_set_ext() || self.instanceof_set_int()
    }

    /// 🆕用于判断是否为「外延交」
    /// * 📄OpenNARS`instanceof IntersectionExt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_intersection_ext(&self) -> bool {
        self.identifier == INTERSECTION_EXT_OPERATOR
    }

    /// 🆕用于判断是否为「内涵交」
    /// * 📄OpenNARS`instanceof IntersectionInt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_intersection_int(&self) -> bool {
        self.identifier == INTERSECTION_INT_OPERATOR
    }

    /// 🆕用于判断是否为「词项交集」
    /// * 📄OpenNARS`instanceof IntersectionExt || instanceof IntersectionInt`逻辑
    /// * 🎯首次用于[`crate::inference::StructuralRules::__switch_order`]
    #[inline(always)]
    pub fn instanceof_intersection(&self) -> bool {
        self.instanceof_intersection_ext() || self.instanceof_intersection_int()
    }

    /// 🆕用于判断是否为「外延差」
    /// * 📄OpenNARS`instanceof DifferenceExt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_difference_ext(&self) -> bool {
        self.identifier == DIFFERENCE_EXT_OPERATOR
    }

    /// 🆕用于判断是否为「内涵差」
    /// * 📄OpenNARS`instanceof DifferenceInt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_difference_int(&self) -> bool {
        self.identifier == DIFFERENCE_INT_OPERATOR
    }

    /// 🆕用于判断是否为「词项差集」
    /// * 📄OpenNARS`instanceof DifferenceExt || instanceof DifferenceInt`逻辑
    #[inline(always)]
    pub fn instanceof_difference(&self) -> bool {
        self.instanceof_difference_ext() || self.instanceof_difference_int()
    }

    /// 🆕用于判断是否为「乘积」
    /// * 📄OpenNARS`instanceof Product`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_product(&self) -> bool {
        self.identifier == PRODUCT_OPERATOR
    }

    /// 🆕用于判断是否为「外延像」
    /// * 📄OpenNARS`instanceof ImageExt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_image_ext(&self) -> bool {
        self.identifier == IMAGE_EXT_OPERATOR
    }

    /// 🆕用于判断是否为「内涵像」
    /// * 📄OpenNARS`instanceof ImageInt`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_image_int(&self) -> bool {
        self.identifier == IMAGE_INT_OPERATOR
    }

    /// 🆕用于判断是否为「像」
    /// * 📄OpenNARS`instanceof ImageExt || instanceof ImageInt`逻辑
    #[inline(always)]
    pub fn instanceof_image(&self) -> bool {
        self.instanceof_image_ext() || self.instanceof_image_int()
    }

    /// 🆕用于判断是否为「合取」
    /// * 📄OpenNARS`instanceof Conjunction`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_conjunction(&self) -> bool {
        self.identifier == CONJUNCTION_OPERATOR
    }

    /// 🆕用于判断是否为「析取」
    /// * 📄OpenNARS`instanceof Disjunction`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_disjunction(&self) -> bool {
        self.identifier == DISJUNCTION_OPERATOR
    }
    /// 🆕用于判断是否为「词项差集」
    /// * 📄OpenNARS`instanceof Conjunction || instanceof Disjunction`逻辑
    #[inline(always)]
    pub fn instanceof_junction(&self) -> bool {
        self.instanceof_conjunction() || self.instanceof_disjunction()
    }

    /// 🆕用于判断是否为「否定」
    /// * 📄OpenNARS`instanceof Negation`逻辑
    /// * 🎯[`crate::inference`]推理规则分派
    #[inline(always)]
    pub fn instanceof_negation(&self) -> bool {
        self.identifier == NEGATION_OPERATOR
    }

    /// 📄OpenNARS `CompoundTerm.isCommutative`
    /// * 📌对「零元/一元 词项」默认为「不可交换」
    ///   * 📜返回`false`
    ///   * 📄OpenNARS中`Negation`的定义（即默认「不可交换」）
    ///
    /// # 📄OpenNARS
    ///
    /// Check if the order of the components matters
    ///
    /// Commutative CompoundTerms: Sets, Intersections
    /// Commutative Statements: Similarity, Equivalence (except the one with a temporal order)
    /// Commutative CompoundStatements: Disjunction, Conjunction (except the one with a temporal order)
    pub fn is_commutative(&self) -> bool {
        matches!(
            self.identifier.as_str(),
            // Commutative CompoundTerms
            SET_EXT_OPERATOR
                | SET_INT_OPERATOR
                | INTERSECTION_EXT_OPERATOR
                | INTERSECTION_INT_OPERATOR
                // Commutative Statements
                | SIMILARITY_RELATION
                | EQUIVALENCE_RELATION
                // Commutative CompoundStatements
                | DISJUNCTION_OPERATOR
                | CONJUNCTION_OPERATOR
        )
    }

    /// 判断和另一词项是否「结构匹配」
    /// * 🎯变量替换中的模式匹配
    /// * 🚩类型匹配 & 组分匹配
    /// * ⚠️非递归：不会递归比较「组分是否对应匹配」
    #[inline(always)]
    pub fn structural_match(&self, other: &Self) -> bool {
        self.is_same_type(other)
        // * 🚩内部组分的「结构匹配」而非自身匹配
            && self
                .components
                .structural_match(&other.components)
    }

    /// 🆕判断是否真的是「复合词项」
    /// * 🚩通过判断「内部元素枚举」的类型实现
    /// * 🎯用于后续「作为复合词项」使用
    ///   * ✨以此在程序层面表示「复合词项」类型
    pub fn is_compound(&self) -> bool {
        matches!(self.components, TermComponents::Compound(..))
    }

    /// 🆕尝试将词项作为「复合词项」
    /// * 📌通过判断「内部元素枚举」的类型实现
    /// * 🚩在其内部元素不是「复合词项」时，会返回`None`
    #[must_use]
    pub fn as_compound(&self) -> Option<CompoundTermRef> {
        matches_or!(
            ?self.components,
            TermComponents::Compound(ref c) => CompoundTermRef {
                inner: self,
                components: c
            }
        )
    }

    /// 🆕尝试将词项作为「复合词项」
    /// * 📌通过判断「内部元素枚举」的类型实现
    /// * 🚩在其内部元素不是「复合词项」时，会返回`None`
    #[must_use]
    pub fn as_compound_and(
        &self,
        predicate: impl FnOnce(&CompoundTermRef) -> bool,
    ) -> Option<CompoundTermRef> {
        match self.as_compound() {
            Some(compound) if predicate(&compound) => Some(compound),
            _ => None,
        }
    }

    /// 🆕尝试将词项作为「复合词项」（未检查）
    /// * 🚩通过判断「内部元素枚举」的类型实现
    ///
    /// # Safety
    ///
    /// * ⚠️代码是不安全的：必须在解包前已经假定是「复合词项」
    /// * 📄逻辑参考自[`Option::unwrap_unchecked`]
    #[must_use]
    pub unsafe fn as_compound_unchecked(&self) -> CompoundTermRef {
        // * 🚩在debug模式下检查
        debug_assert!(self.is_compound(), "转换前必须假定其为复合词项");
        // * 🚩正式开始解引用
        match self.components {
            TermComponents::Compound(ref c) => CompoundTermRef {
                inner: self,
                components: c,
            },
            // SAFETY: the safety contract must be upheld by the caller.
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// 🆕尝试将词项作为「复合词项」
    /// * ℹ️[`Self::as_compound`]的可变版本
    #[must_use]
    pub fn as_compound_mut(&mut self) -> Option<CompoundTermRefMut> {
        matches_or! {
            // * 📌此处需要可变借用，才能在下头正常把Box变成可变引用（而无需Deref）
            // * ❌使用`ref mut`不能达到目的：解引用后还是Box
            ?&mut self.components,
            TermComponents::Compound(components) => CompoundTermRefMut {
                // * 🚩【2024-06-15 14:00:09】此处创建裸指针，是安全行为（解引用才是不安全行为）
                // * 📄具体使用参见[`CompoundTermRefMut::components`]
                components: &mut **components as *mut [Term],
                inner   :self,
            }
        }
    }

    /// 🆕尝试将词项作为「可变复合词项」（未检查）
    /// * 🚩通过判断「内部元素枚举」的类型实现
    ///
    /// # Safety
    ///
    /// * ⚠️代码是不安全的：必须在解包前已经假定是「复合词项」
    /// * 📄逻辑参考自[`Option::unwrap_unchecked`]
    #[must_use]
    pub unsafe fn as_compound_mut_unchecked(&mut self) -> CompoundTermRefMut {
        // * 🚩在debug模式下检查
        debug_assert!(self.is_compound(), "转换前必须假定其为复合词项");
        // * 🚩正式开始解引用
        match &mut self.components {
            TermComponents::Compound(components) => CompoundTermRefMut {
                // * 🚩【2024-06-15 14:00:09】此处创建裸指针，是安全行为（解引用才是不安全行为）
                // * 📄具体使用参见[`CompoundTermRefMut::components`]
                components: &mut **components as *mut [Term],
                inner: self,
            },
            // SAFETY: the safety contract must be upheld by the caller.
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

/// 从NAL语义上判断词项的「容量」
impl GetCapacity for Term {
    fn get_capacity(&self) -> TermCapacity {
        use TermCapacity::*;
        match self.identifier.as_str() {
            // * 🚩原子：词语、占位符、变量
            WORD | PLACEHOLDER | VAR_INDEPENDENT | VAR_DEPENDENT | VAR_QUERY => Atom,
            // * 🚩一元：否定
            NEGATION_OPERATOR => Unary,
            // * 🚩二元序列：差集、继承、蕴含、预测性蕴含 | ❌不包括「实例」「属性」「实例属性」
            DIFFERENCE_EXT_OPERATOR
            | DIFFERENCE_INT_OPERATOR
            | INHERITANCE_RELATION
            | IMPLICATION_RELATION
            | PREDICTIVE_IMPLICATION_RELATION => BinaryVec,
            // * 🚩二元集合：相似、等价
            SIMILARITY_RELATION | EQUIVALENCE_RELATION => BinarySet,
            // * 🚩多元序列：乘积、像、序列合取
            PRODUCT_OPERATOR
            | IMAGE_EXT_OPERATOR
            | IMAGE_INT_OPERATOR
            | SEQUENTIAL_CONJUNCTION_OPERATOR => Vec,
            // * 🚩多元集合：词项集、交集、合取、析取、平行合取
            SET_EXT_OPERATOR
            | SET_INT_OPERATOR
            | INTERSECTION_EXT_OPERATOR
            | INTERSECTION_INT_OPERATOR
            | CONJUNCTION_OPERATOR
            | DISJUNCTION_OPERATOR
            | PARALLEL_CONJUNCTION_OPERATOR => Set,
            // * 🚩其它⇒panic（不应出现）
            _ => panic!("Unexpected compound term identifier: {}", self.identifier),
        }
    }
}

/// 🆕作为「复合词项引用」的词项类型
/// * 🎯在程序类型层面表示一个「复合词项」（不可变引用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundTermRef<'a> {
    /// 复合词项整体
    pub inner: &'a Term,
    /// 复合词项的元素列表
    pub components: &'a [Term],
}

impl<'s> CompoundTermRef<'s> {
    /// 📄OpenNARS `CompoundTerm.size`
    /// * 🚩直接链接到[`TermComponents`]的属性
    /// * ⚠️对「像」不包括「像占位符」
    ///   * 📄`(/, A, _, B)`的`size`为`2`而非`3`
    ///
    /// # 📄OpenNARS
    ///
    /// get the number of components
    #[inline]
    pub fn size(&self) -> usize {
        self.components.len()
    }

    /// 📄OpenNARS `CompoundTerm.componentAt`
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * ⚠️对「像」不受「像占位符」位置影响
    ///
    /// # 📄OpenNARS
    ///
    /// get a component by index
    #[inline]
    pub fn component_at(self, index: usize) -> Option<&'s Term> {
        self.components.get(index)
    }

    /// 📄OpenNARS `CompoundTerm.componentAt`
    /// * 🆕unsafe版本：若已知词项的组分数，则可经此对症下药
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * ⚠️对「像」不受「像占位符」位置影响
    ///
    /// # Safety
    ///
    /// ⚠️只有在「确保索引不会越界」才不会引发panic
    ///
    /// # 📄OpenNARS
    ///
    /// get a component by index
    #[inline]
    pub unsafe fn component_at_unchecked(&self, index: usize) -> &Term {
        self.components.get_unchecked(index)
    }

    /// 📄OpenNARS `CompoundTerm.getComponents`
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * 🚩【2024-04-21 16:11:59】目前只需不可变引用
    ///   * 🔎OpenNARS中大部分用法是「只读」情形
    /// * 🚩自改版：仅在复合词项「移除元素」时使用
    ///
    /// # 📄OpenNARS
    ///
    /// Get the component list
    #[inline]
    pub(super) fn get_components(&self) -> impl Iterator<Item = &Term> {
        self.components.iter()
    }

    /// 🆕改版 `CompoundTerm.indexOfComponent`
    ///
    /// @param t [&]
    /// @return [] index or -1
    ///
    pub fn index_of_component(&self, t: &Term) -> Option<usize> {
        self.components.iter().position(|term| term == t)
    }

    /// 📄OpenNARS `CompoundTerm.cloneComponents`
    /// * 🚩【2024-06-14 10:43:03】遵照改版原意，使用变长数组
    ///   * ℹ️后续需要增删操作
    ///   * 📝无论如何也绕不开[`Vec`]
    ///
    /// # 📄OpenNARS
    ///
    /// Clone the component list
    pub fn clone_components(&self) -> Vec<Term> {
        self.components.to_vec()
    }

    /// 📄OpenNARS `CompoundTerm.cloneComponents`
    /// * 🚩只拷贝所有元素的引用，无需拷贝其中的值
    pub fn clone_component_refs(&self) -> Vec<&Term> {
        self.components.iter().collect()
    }

    /// 📄OpenNARS `CompoundTerm.containComponent`
    /// * 🎯检查其是否包含**直接**组分
    /// * 🚩直接基于已有迭代器方法
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the compound contains a certain component
    pub fn contain_component(&self, component: &Term) -> bool {
        self.get_components().any(|term| term == component)
    }

    /// 📄OpenNARS `CompoundTerm.containTerm`
    /// * 🎯检查其是否**递归**包含组分
    /// * 🚩直接基于已有迭代器方法：词项 == 组分 || 词项 in 组分
    ///
    /// # 📄OpenNARS
    ///
    /// Recursively check if a compound contains a term
    pub fn contain_term(&self, term: &Term) -> bool {
        self.get_components()
            .any(|sub_term| match sub_term.as_compound() {
                // * 🚩非复合⇒判等
                None => term == sub_term,
                // * 🚩复合⇒递归
                Some(sub_compound) => sub_compound.contain_term(term),
            })
    }

    /// 📄OpenNARS `CompoundTerm.containAllComponents`
    /// * 🎯分情况检查「是否包含所有组分」
    ///   * 📌同类⇒检查其是否包含`other`的所有组分
    ///   * 📌异类⇒检查其是否包含`other`作为整体
    /// * 🚩直接基于已有迭代器方法
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the compound contains all components of another term, or that term as a whole
    pub fn contain_all_components(&self, other: &Term) -> bool {
        match self.inner.is_same_type(other) {
            // * 🚩再判断内层是否为复合词项
            true => match other.as_compound() {
                // * 🚩复合词项⇒深入一层
                Some(other) => other
                    .get_components()
                    .all(|should_in| self.contain_component(should_in)),
                _ => false,
            },
            false => self.contain_component(other),
        }
    }

    /// 🆕作为「条件句」使用
    /// * 🚩转发到[「陈述」](StatementRef::as_conditional)中
    ///
    /// ! ❌【2024-07-05 17:04:02】不再考虑支持「等价」陈述的词项链转换，同时也不再将「等价陈述」视作「条件句」
    pub fn as_conditional(self) -> Option<(StatementRef<'s>, CompoundTermRef<'s>)> {
        self.as_statement()?.as_conditional()
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for CompoundTermRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「复合词项引用」可以被看作是一个普通的词项
impl Deref for CompoundTermRef<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// 🆕作为「复合词项引用」的词项类型
/// * 🎯在程序类型层面表示一个「复合词项」（可变引用）
/// * ⚠️取舍：因可变引用无法共享，此时需要在构造层面限制
///   * 📌构造时保证「内部组分」为「复合词项」变种
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CompoundTermRefMut<'a> {
    /// 复合词项内部的词项整体（自身）
    pub(super) inner: &'a mut Term,
    /// 复合词项内部的元素列表
    /// * ⚠️【2024-06-15 13:45:47】尝试使用裸指针，不安全代码封装安全接口
    pub(super) components: *mut [Term],
}

impl CompoundTermRefMut<'_> {
    /// 获取词项整体
    pub fn inner(&mut self) -> &mut Term {
        self.inner
    }

    /// 获取内部组分
    /// * 📌【2024-06-15 14:56:33】需要用可变引用`&mut self`保证「独占性」
    ///
    /// # Panics
    ///
    /// ! ⚠️若使用了非法的构造方式将「非复合词项」构造入此，则将抛出panic
    pub fn components(&mut self) -> &mut [Term] {
        // matches_or!(
        //     self.inner.components,
        //     TermComponents::Compound(ref mut components) => components,
        //     unreachable!("CompoundTermRefMut::components 断言失败：不是复合词项: {}", self.inner)
        // )
        // * ✅即：不可能在「调用components」与「使用components」之间插入「inner」
        // * 🚩解引用前（在debug模式下）检查
        debug_assert!(self.inner.is_compound());
        // * 🚩解引用
        // ! SAFETY: 此处保证对整体（整个复合词项）拥有引用
        unsafe { &mut *self.components }
    }

    /// 生成一个不可变引用
    /// * 🚩将自身的所有字段转换为不可变引用，然后构造一个「不可变引用」结构
    /// * 📌可变引用一定能转换成不可变引用
    /// * ⚠️与[`AsRef`]与[`Deref`]不同：此处需要返回所有权，而非对目标类型（[`Term`]）的引用
    ///   * ❌返回`&CompoundTermRef`会导致「返回临时变量引用」故无法使用
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    pub fn into_ref<'s>(self) -> CompoundTermRef<'s>
    where
        Self: 's,
    {
        // * 🚩解引用前（在debug模式下）检查
        debug_assert!(self.inner.is_compound());
        // * 🚩传递引用 & 裸指针解引用
        CompoundTermRef {
            inner: self.inner,
            // SAFETY: 自身相当于对词项的可变引用，同时两个字段均保证有效——那就一定能同时转换
            components: unsafe { &*self.components },
        }
    }

    /* ----- variable-related utilities ----- */

    // ! 📌`set_term_when_dealing_variables`现在不再使用：直接在「变量处理」中设置指针所指向的值

    /// 🆕对于「可交换词项」重排其中的元素
    /// * 🚩【2024-06-13 18:05:40】只在「应用替换」时用到
    /// * 🚩【2024-06-14 13:37:46】使用「内存交换」魔法代码
    /// * 🚩包含「排序」「去重」两个作用
    pub fn reorder_components(&mut self) {
        // * 🚩构造一个「占位符」并将其与已有组分互换
        let mut placeholder = TermComponents::Empty;
        std::mem::swap(&mut placeholder, &mut self.inner.components);
        // * 🚩将替换后名为「占位符」的实际组分进行「重排去重」得到「新组分」
        let new_components = placeholder.sort_dedup();
        // * 🚩将「新组分」赋值回原先的组分，原先位置上的「占位符」被覆盖
        self.inner.components = new_components;
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for CompoundTermRefMut<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「复合词项可变引用」可以被看作是一个普通的词项
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
/// * ❌【2024-06-15 15:38:58】不能实现「自动解引用到不可变引用」
impl Deref for CompoundTermRefMut<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「复合词项可变引用」可以被看作是一个普通的词项（可变引用）
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
impl DerefMut for CompoundTermRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

/// 可变引用 ⇒ 不可变引用
impl<'s> From<CompoundTermRefMut<'s>> for CompoundTermRef<'s> {
    fn from(r: CompoundTermRefMut<'s>) -> Self {
        r.into_ref()
    }
}

/// 具备所有权的复合词项
/// * 🎯初步决定用于「推理规则」向下分派
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompoundTerm {
    /// 内部词项
    term: Term,
}

impl CompoundTerm {
    /// 获取不可变引用
    pub fn get_ref(&self) -> CompoundTermRef {
        // SAFETY: 在构造时，已经检查了是否为复合词项，因此此处无需检查
        unsafe { self.term.as_compound_unchecked() }
    }

    /// 获取可变引用
    pub fn mut_ref(&mut self) -> CompoundTermRefMut {
        // SAFETY: 在构造时，已经检查了是否为复合词项，因此此处无需检查
        unsafe { self.term.as_compound_mut_unchecked() }
    }
}

/// 仅有的一处入口：从[词项](Term)构造
impl TryFrom<Term> for CompoundTerm {
    /// 转换失败时，返回原始词项
    type Error = Term;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        // * 🚩仅在是复合词项时转换成功
        match term.is_compound() {
            true => Ok(Self { term }),
            false => Err(term),
        }
    }
}

/// 出口（转换成词项）
impl From<CompoundTerm> for Term {
    fn from(value: CompoundTerm) -> Self {
        value.term
    }
}

/// 方便直接作为词项使用
/// * ❓是否要滥用此种「类似继承的模式」
impl Deref for CompoundTerm {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        &self.term
    }
}

/// 方便直接作为词项使用（可变）
impl DerefMut for CompoundTerm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.term
    }
}

/// 内联「显示呈现」
impl Display for CompoundTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.term.fmt(f)
    }
}

/// 单元测试
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::{asserts, macro_once};

    /// 构建测试用复合词项
    #[macro_export]
    macro_rules! test_compound {
        // 具所有权
        (box $($t:tt)*) => {
            CompoundTerm::try_from(term!($($t)*)).unwrap()
        };
        // 可变
        (mut $($t:tt)*) => {
            term!($($t)*).as_compound_mut().unwrap()
        };
        // 不可变
        ($($t:tt)*) => {
            term!($($t)*).as_compound().unwrap()
        };
    }

    /// 转发，用于模块内部
    /// * ❌【2024-06-16 13:44:19】无法在内部use
    macro_rules! compound {
        ($($t:tt)*) => {
            test_compound!($($t)*)
        };
    }

    /// 「词项」与「复合词项」相关的代码
    mod term {
        use super::*;

        #[test]
        fn instanceof_compound() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro instanceof_compound($( $s:literal => $expected:expr )*) {
                    asserts! {$(
                        term!($s).instanceof_compound() => $expected,
                    )*}
                }
                // 占位符
                "_" => false
                // 原子词项
                "A" => false
                "$A" => false
                "#A" => false
                "?A" => false
                // 复合词项
                "{A}" => true
                "[A]" => true
                "(&, A)" => true
                "(|, A)" => true
                "(-, A, B)" => true
                "(~, A, B)" => true
                "(*, A)" => true
                r"(/, R, _)" => true
                r"(\, R, _)" => true
                r"(&&, A)" => true
                r"(||, A)" => true
                r"(--, A)" => true
                // 陈述
                "<A --> B>" => true
                "<A <-> B>" => true
                "<A ==> B>" => true
                "<A <=> B>" => true
            }
            ok!()
        }

        #[test]
        fn is_commutative() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro is_commutative($( $s:literal => $expected:expr )*) {
                    asserts! {$(
                        term!($s).is_commutative() => $expected,
                    )*}
                }
                // 占位符
                "_" => false
                // 原子词项
                "A" => false
                "$A" => false
                "#A" => false
                "?A" => false
                // 复合词项
                "{A}" => true
                "[A]" => true
                "(&, A)" => true
                "(|, A)" => true
                "(-, A, B)" => false
                "(~, A, B)" => false
                "(*, A)" => false
                r"(/, R, _)" => false
                r"(\, R, _)" => false
                r"(&&, A)" => true
                r"(||, A)" => true
                r"(--, A)" => false
                // 陈述
                "<A --> B>" => false
                "<A <-> B>" => true
                "<A ==> B>" => false
                "<A <=> B>" => true
            }
            ok!()
        }
    }

    /// 复合词项不可变引用
    mod compound_term_ref {
        use super::*;

        #[test]
        fn deref() -> AResult {
            /// 通用测试函数
            fn test(term: Term) {
                // * 🚩首先是一个复合词项
                assert!(term.is_compound());
                // * 🚩无检查转换到复合词项（不可变引用）
                let compound = unsafe { term.as_compound_unchecked() };
                // * 🚩像一个普通的词项（不可变引用）使用
                dbg!(compound.identifier(), compound.components());

                // * 🚩安全：可被多次共用
                let c1 = compound; // ! Copy特征无需显式clone
                let c2 = compound.as_compound().unwrap();
                let c3 = term.as_compound().unwrap();
                dbg!(c1, c2, c3); // 同时出现

                // * 🚩其它系列特性
                asserts! {
                    compound.is_compound(),
                    compound.as_compound() => Some(compound),
                    // * 📌还可以使用：因为CompoundTermRef实现了Copy特征
                    *compound => term, // ! 这毕竟是引用，需要解引用才能
                    compound.clone() => compound, // ! 引用的复制≠自身的复制
                    (*compound).clone() => term, // ! 解引用后复制，结果才相等
                }
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(term!($term));
                )*}
                // // 占位符
                // "_" => 0
                // // 原子词项
                // "A" => 0
                // "$A" => 0
                // "#A" => 0
                // "?A" => 0
                // 复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A, B, C)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        #[test]
        fn size() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro size($( $s:literal => $expected:expr )*) {
                    asserts! {$(
                        compound!($s).size() => $expected,
                    )*}
                }
                // // 占位符
                // "_" => 0
                // // 原子词项
                // "A" => 0
                // "$A" => 0
                // "#A" => 0
                // "?A" => 0
                // 复合词项
                "{A}" => 1
                "[A]" => 1
                "(&, A)" => 1
                "(|, A)" => 1
                "(-, A, B)" => 2
                "(~, A, B)" => 2
                "(*, A, B, C)" => 3
                r"(/, R, _)" => 2 // * ⚠️算入占位符
                r"(\, R, _)" => 2
                r"(&&, A)" => 1
                r"(||, A)" => 1
                r"(--, A)" => 1
                // 陈述
                "<A --> B>" => 2
                "<A <-> B>" => 2
                "<A ==> B>" => 2
                "<A <=> B>" => 2
            }
            ok!()
        }

        #[test]
        fn component_at() -> AResult {
            // 命中
            macro_once! {
                // * 🚩模式：词项字符串[索引] ⇒ 预期词项
                macro component_at($( $s:literal [ $index:expr ] => $expected:expr )*) {
                    asserts! {$(
                        compound!($s).component_at($index) => Some(&term!($expected)),
                    )*}
                }
                // 复合词项
                "{A}"[0] => "A"
                "[A]"[0] => "A"
                "(&, A)"[0] => "A"
                "(|, A)"[0] => "A"
                "(-, A, B)"[1] => "B"
                "(~, A, B)"[1] => "B"
                "(*, A, B, C)"[2] => "C"
                r"(/, R, _)"[0] => "R" // * ⚠️算入占位符
                r"(\, R, _)"[0] => "R"
                r"(/, R, _)"[1] => "_" // * ⚠️算入占位符
                r"(\, R, _)"[1] => "_"
                r"(&&, A)"[0] => "A"
                r"(||, A)"[0] => "A"
                r"(--, A)"[0] => "A"
                // 陈述
                "<A --> B>"[0] => "A"
                "<A <-> B>"[0] => "A"
                "<A ==> B>"[0] => "A"
                "<A <=> B>"[0] => "A"
            }
            // 未命中
            macro_once! {
                // * 🚩模式：词项字符串[索引]
                macro component_at($( $s:literal [ $index:expr ] )*) {
                    asserts! {$(
                        compound!($s).component_at($index) => None,
                    )*}
                }
                // // 占位符
                // "_"[0]
                // // 原子词项
                // "A"[0]
                // "$A"[0]
                // "#A"[0]
                // "?A"[0]
                // 复合词项
                "{A}"[1]
                "[A]"[1]
                "(&, A)"[1]
                "(|, A)"[1]
                "(-, A, B)"[2]
                "(~, A, B)"[2]
                "(*, A, B, C)"[3]
                r"(/, R, _)"[2] // * ⚠️算入占位符
                r"(\, R, _)"[2]
                r"(&&, A)"[1]
                r"(||, A)"[1]
                r"(--, A)"[1]
                // 陈述
                "<A --> B>"[2]
                "<A <-> B>"[2]
                "<A ==> B>"[2]
                "<A <=> B>"[2]
            }
            ok!()
        }

        #[test]
        fn component_at_unchecked() -> AResult {
            // 命中
            macro_once! {
                // * 🚩模式：词项字符串[索引] ⇒ 预期词项
                macro component_at_unchecked($( $s:literal [ $index:expr ] => $expected:expr )*) {
                    unsafe {
                        asserts! {$(
                            compound!($s).component_at_unchecked($index) => &term!($expected),
                        )*}
                    }
                }
                // 复合词项
                "{A}"[0] => "A"
                "[A]"[0] => "A"
                "(&, A)"[0] => "A"
                "(|, A)"[0] => "A"
                "(-, A, B)"[1] => "B"
                "(~, A, B)"[1] => "B"
                "(*, A, B, C)"[2] => "C"
                r"(/, R, _)"[0] => "R" // ! 不算占位符
                r"(\, R, _)"[0] => "R"
                r"(&&, A)"[0] => "A"
                r"(||, A)"[0] => "A"
                r"(--, A)"[0] => "A"
                // 陈述
                "<A --> B>"[0] => "A"
                "<A <-> B>"[0] => "A"
                "<A ==> B>"[0] => "A"
                "<A <=> B>"[0] => "A"
            }
            ok!()
        }

        // * ✅`get_components`已在[`TermComponents::iter`]中测试

        #[test]
        fn clone_components() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 | 复制之后与新词项的「组分」相等
                macro clone_components($($s:literal)*) {
                    asserts! {$(
                        // * 🚩假设其拷贝的词项与迭代器收集的相等
                        compound!($s).clone_components() => term!($s).components.iter().cloned().collect::<Vec<_>>(),
                    )*}
                }
                // // 占位符
                // "_"
                // // 原子词项
                // "A"
                // "$A"
                // "#A"
                // "?A"
                // 复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        #[test]
        fn contain_component() -> AResult {
            macro_once! {
                // * 🚩模式：词项 in 容器词项
                macro contain_component($($term:literal in $container:expr)*) {
                    asserts! {$(
                        compound!($container).contain_component(&term!($term))
                    )*}
                }
                // 复合词项
                "A" in "{A}"
                "A" in "[A]"
                "A" in "(&, A)"
                "A" in "(|, A)"
                "A" in "(-, A, B)"
                "A" in "(~, A, B)"
                "B" in "(-, A, B)"
                "B" in "(~, A, B)"
                "A" in "(*, A)"
                "R" in r"(/, R, _)"
                "R" in r"(\, R, _)"
                "_" in r"(/, R, _)" // ! 📌【2024-06-14 13:46:19】现在「占位符」也包含在内
                "_" in r"(\, R, _)" // ! 📌【2024-06-14 13:46:19】现在「占位符」也包含在内
                "A" in r"(&&, A)"
                "A" in r"(||, A)"
                "A" in r"(--, A)"
                // 陈述
                "A" in "<A --> B>"
                "A" in "<A <-> B>"
                "A" in "<A ==> B>"
                "A" in "<A <=> B>"
                "B" in "<A --> B>"
                "B" in "<A <-> B>"
                "B" in "<A ==> B>"
                "B" in "<A <=> B>"
            }
            macro_once! {
                // * 🚩模式：词项 in 容器词项
                macro contain_component($($term:literal !in $container:expr)*) {
                    asserts! {$(
                        !compound!($container).contain_component(&term!($term))
                    )*}
                }
                // 复合词项
                "X" !in "{A}"
                "X" !in "[A]"
                "X" !in "(&, A)"
                "X" !in "(|, A)"
                "X" !in "(-, A, B)"
                "X" !in "(~, A, B)"
                "X" !in "(*, A)"
                "X" !in r"(/, R, _)"
                "X" !in r"(\, R, _)"
                "X" !in r"(&&, A)"
                "X" !in r"(||, A)"
                "X" !in r"(--, A)"
                // 陈述
                "C" !in "<A --> B>"
                "C" !in "<A <-> B>"
                "C" !in "<A ==> B>"
                "C" !in "<A <=> B>"
            }
            ok!()
        }

        #[test]
        fn contain_term() -> AResult {
            macro_once! {
                // * 🚩模式：词项 in 容器词项
                macro contain_term($($term:literal in $container:expr)*) {
                    asserts! {$(
                        compound!($container).contain_term(&term!($term))
                    )*}
                }
                // 复合词项
                "A" in "{{{{{{A}}}}}}"
                "A" in "[[[[[[A]]]]]]"
                "A" in "(&, (&, (&, (&, (&, A)))))"
                "A" in "(|, (|, (|, (|, (|, A)))))"
                "A" in "(-, (-, A, a), (-, B, b))"
                "A" in "(~, (~, A, a), (~, B, b))"
                "B" in "(-, (-, A, a), (-, B, b))"
                "B" in "(~, (~, A, a), (~, B, b))"
                "A" in "(*, (*, (*, (*, (*, A)))))"
                "R" in r"(/, (/, (/, (/, (/, R, _), _), _), _), _)"
                "R" in r"(\, (\, (\, (\, (\, R, _), _), _), _), _)"
                "A" in r"(&&, (&&, (&&, (&&, (&&, A)))))"
                "A" in r"(||, (||, (||, (||, (||, A)))))"
                "A" in r"(--, (--, (--, (--, (--, A)))))"
                // 陈述
                "A" in "<<A --> a> --> <B --> b>>"
                "B" in "<<A --> a> --> <B --> b>>"
                "A" in "<<A <-> a> <-> <B <-> b>>"
                "B" in "<<A <-> a> <-> <B <-> b>>"
                "A" in "<<A ==> a> ==> <B ==> b>>"
                "B" in "<<A ==> a> ==> <B ==> b>>"
                "A" in "<<A <=> a> <=> <B <=> b>>"
                "B" in "<<A <=> a> <=> <B <=> b>>"
            }
            macro_once! {
                // * 🚩模式：词项 in 容器词项
                macro contain_term($($term:literal !in $container:expr)*) {
                    asserts! {$(
                        !compound!($container).contain_term(&term!($term))
                    )*}
                }
                // 复合词项
                "X" !in "{{{{{{A}}}}}}"
                "X" !in "[[[[[[A]]]]]]"
                "X" !in "(&, (&, (&, (&, (&, A)))))"
                "X" !in "(|, (|, (|, (|, (|, A)))))"
                "X" !in "(-, (-, A, a), (-, B, b))"
                "X" !in "(~, (~, A, a), (~, B, b))"
                "X" !in "(*, (*, (*, (*, (*, A)))))"
                "X" !in r"(/, (/, (/, (/, (/, R, _), _), _), _), _)"
                "X" !in r"(\, (\, (\, (\, (\, R, _), _), _), _), _)"
                "X" !in r"(&&, (&&, (&&, (&&, (&&, A)))))"
                "X" !in r"(||, (||, (||, (||, (||, A)))))"
                "X" !in r"(--, (--, (--, (--, (--, A)))))"
                // 陈述
                "X" !in "<<A --> a> --> <B --> b>>"
                "X" !in "<<A <-> a> <-> <B <-> b>>"
                "X" !in "<<A ==> a> ==> <B ==> b>>"
                "X" !in "<<A <=> a> <=> <B <=> b>>"
            }
            ok!()
        }

        #[test]
        fn contain_all_components() -> AResult {
            macro_once! {
                // * 🚩模式：词项 in 容器词项
                macro test($($term:literal in $container:expr)*) {
                    asserts! {$(
                        compound!($container).contain_all_components(&term!($term))
                    )*}
                }
                // 复合词项
                "A" in "{A}"
                "{A}" in "{A}"
                "{A}" in "{A, B}"
                "{A}" in "{A, B, C}"
                "{B}" in "{A, B, C}"
                "{C}" in "{A, B, C}"
                "{A, B}" in "{A, B, C}"
                "{A, C}" in "{A, B, C}"
                "{B, C}" in "{A, B, C}"
                "{A, B, C}" in "{A, B, C}"
                "A" in "(-, A, B)"
                "B" in "(-, A, B)"
                "(-, A, B)" in "(-, A, B)"
                "A" in "(*, A, B, C, D, E)"
                "(*, A)" in "(*, A, B, C, D, E)"
                "(*, A, B)" in "(*, A, B, C, D, E)"
                "(*, E, B)" in "(*, A, B, C, D, E)"
                "(*, E, A)" in "(*, A, B, C, D, E)"
                "R" in r"(/, R, _)"
                "_" in r"(/, R, _)"
                "R" in r"(/, R, _, (*, A))"
                "_" in r"(/, R, _, (*, A))"
                "(*, A)" in r"(/, R, _, (*, A))"
                r"(/, R, _)" in r"(/, R, _, (*, A))"
                "R" in r"(\, R, _)"
                "_" in r"(\, R, _)"
                "R" in r"(\, R, _, (*, A))"
                "_" in r"(\, R, _, (*, A))"
                "(*, A)" in r"(\, R, _, (*, A))"
                r"(\, R, _)" in r"(\, R, _, (*, A))"
                // 陈述
                "A" in "<A --> B>"
                "B" in "<A --> B>"
                "<A --> B>" in "<A --> B>"
                "<B --> A>" in "<A --> B>"
                "A" in "<A <-> B>"
                "B" in "<A <-> B>"
                "<A <-> B>" in "<A <-> B>"
                "<B <-> A>" in "<A <-> B>"
                "A" in "<A ==> B>"
                "B" in "<A ==> B>"
                "<A ==> B>" in "<A ==> B>"
                "<B ==> A>" in "<A ==> B>"
                "A" in "<A <=> B>"
                "B" in "<A <=> B>"
                "<A <=> B>" in "<A <=> B>"
                "<B <=> A>" in "<A <=> B>"
            }
            ok!()
        }
    }

    /// 复合词项可变引用
    mod compound_term_ref_mut {
        use super::*;

        /// 保证整个接口是安全的
        #[test]
        #[allow(unused_variables)]
        pub fn assure_safe_interface() -> AResult {
            fn use_inner(_: &mut Term) {}
            fn use_components(_: &mut [Term]) {}
            let mut term = term!("(*, A, B, C)");
            let mut mut_compound = term.as_compound_mut().expect("无法转换为可变复合词项");

            // 先用元素集合，再用词项自身
            let components = mut_compound.components();
            let inner = mut_compound.inner();
            // ! 在这之后是用不了`components`的：因为`inner`已经借走了`mut_compound`的引用
            // * 📝实际上`components`的生命周期早已在`inner`处结束，只是因为「自动作用域调整」才【显得】可以共存
            // use_terms(components);
            use_inner(inner);
            // * ✅下面这个是被允许的：有方式保证inner与整体不会同时出现，那就是让inner生命期在这之前结束
            use_components(mut_compound.components());
            // drop(inner); // ! 在这之后同样用不了`inner`：不允许整体被同时可变借用两次
            use_inner(mut_compound.inner()); // * ✅这个是被允许的：上头的可变引用创建后就被传入（然后回收）

            // 先用词项自身，再用元素集合
            let inner = mut_compound.inner();
            let components = mut_compound.components();
            // ! 在这之后是用不了`inner`的：因为`components`已经借走了`mut_compound`的引用
            // * 📝实际上`inner`的生命周期早已在`components`处结束，只是因为「自动作用域调整」才【显得】可以共存
            // use_term(inner);
            use_components(components);
            // * ✅下面这个是被允许的：有方式保证inner与整体不会同时出现，那就是让components生命期在这之前结束
            use_inner(mut_compound.inner());
            // drop(components); // ! 在这之后同样用不了`inner`：不允许整体被同时可变借用两次
            use_components(mut_compound.components()); // * ✅这个是被允许的：上头的可变引用创建后就被传入（然后回收）

            // components; // * 📌接下来不再允许使用`components`：中间可变借用了mut_compound，因此生命期被限定在借用之前
            // inner; // * 📌接下来不再允许使用`inner`：中间可变借用了mut_compound，因此生命期被限定在借用之前

            ok!()
        }

        /// 解引用：可变/不可变
        /// * ✅同时测试[`Deref`]与[`DerefMut`]
        #[test]
        fn deref_and_mut() -> AResult {
            /// 通用测试函数
            #[allow(clippy::explicit_auto_deref)]
            fn test(mut term: Term) {
                // * 🚩首先是一个复合词项
                assert!(term.is_compound());
                // * 🚩无检查转换到复合词项（可变引用）
                let term2 = term.clone();
                let mut compound = unsafe { term.as_compound_mut_unchecked() };
                // dbg!(term.as_compound_mut()); // * ✅安全：借用检查拦截了「重复借用」行为

                // * 🚩像一个普通的词项（不可变引用）使用：一次只能传入一个
                // dbg!(compound.identifier(), compound.components());
                dbg!(compound.identifier());
                dbg!(compound.components());

                // * 🚩像一个普通的词项（可变引用）使用：一次只能传入一个
                dbg!(compound.components_mut());
                let original_id = compound.identifier().to_string();
                compound.identifier = "MUTATED".into(); // * 🚩自动解引用并修改字段
                assert_eq!(compound.identifier, "MUTATED");
                (*compound).identifier = original_id; // * 🚩与上述语法等价，但这次是改回原标识符

                // * 🚩检验潜在风险：使用Deref拷贝出并存的不可变引用
                let compound_ref = compound.as_compound().unwrap();
                // (compound_ref, compound);
                // * ✅安全：生命期约束下，不可变引用与可变引用无法同时存在
                // * 📝在调用`.as_compound()`之后，返回值的生命期即不可变引用的生命期
                // * 📝因此在「得到的不可变引用」生命期结束前，不能使用可变引用
                dbg!(compound_ref, compound_ref, compound_ref); // ! 转换成的不可变引用，可以同时存在多个

                // * 🚩其它属性的验证
                asserts! {
                    compound.is_compound(),
                    compound.as_compound().is_some(),
                    compound.as_compound_mut().is_some(),
                    // ! 可变引用未实现Clone和Copy特征，但因实现了Deref而可以使用clone方法
                    *compound => term2, // ! 这毕竟是引用，需要解引用才能
                    compound.clone() => term2, // ! 引用的复制=自身的复制
                    (*compound).clone() => term2, // ! 解引用后复制，结果仍相等
                }
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(term!($term));
                )*}
                // // 占位符
                // "_" => 0
                // // 原子词项
                // "A" => 0
                // "$A" => 0
                // "#A" => 0
                // "?A" => 0
                // 复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A, B, C)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        #[test]
        pub fn components() -> AResult {
            macro_once! {
                macro test($($term:literal => $container:expr)*) {
                    asserts! {$(
                            compound!(mut $term).components()
                            => $container
                    )*}
                }
                "{A}" => [term!(A)]
                "(--, A)" => [term!(A)]
                "(-, A, B)" => term!(["A", "B"])
                "(~, A, B)" => term!(["A", "B"])
                "{A, B, C}" => term!(["A", "B", "C"])
                "[A, B, C]" => term!(["A", "B", "C"])
                "(*, A, B, C)" => term!(["A", "B", "C"])
                "(/, A, B, C, _)" => term!(["A", "B", "C", "_"])
                "<A --> B>" => term!(["A", "B"])
                "<A <-> B>" => term!(["A", "B"])
                "<A ==> B>" => term!(["A", "B"])
                "<A <=> B>" => term!(["A", "B"])
                "<A --> B>" => term!(["A", "B"])
                "<A <-> B>" => term!(["A", "B"])
                "<A ==> B>" => term!(["A", "B"])
                "<A <=> B>" => term!(["A", "B"])
            }
            ok!()
        }

        #[test]
        pub fn into_ref() -> AResult {
            macro_once! {
                macro test($($term:literal)*) {
                    asserts! {$(
                            compound!(mut $term).into_ref()
                            => compound!($term)
                    )*}
                }
                "{A}"
                "(--, A)"
                "(-, A, B)"
                "(~, A, B)"
                "{A, B, C}"
                "[A, B, C]"
                "(*, A, B, C)"
                "(/, A, B, C, _)"
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        // ! ℹ️【2024-06-19 18:16:10】现在此处直接在特定引用处设置值
        #[test]
        pub fn set_term_when_dealing_variables() -> AResult {
            fn test(mut term: Term, i: usize, new: Term, expected: Term) {
                term.as_compound_mut().unwrap().components()[i] = new;
                assert_eq!(term, expected);
            }
            macro_once! {
                macro test($(
                    $term:literal [$i:expr] = $new:literal =>
                    $expected:literal
                )*) {
                    $( test( term!($term), $i, term!($new), term!($expected)); )*
                }
                "{A}"[0] = "B" => "{B}"
                "(--, A)"[0] = "B" => "(--, B)"
                "(-, A, B)"[0] = "a" => "(-, a, B)"
                "(~, A, B)"[0] = "a" => "(~, a, B)"
                "{A, B, Z}"[1] = "X" => "{A, X, Z}" // ! 集合词项在从字符串解析时会重排，所以不能用`C`
                "[A, B, Z]"[1] = "X" => "[A, X, Z]" // ! 集合词项在从字符串解析时会重排，所以不能用`C`
                "(*, A, B, C)"[1] = "X" => "(*, A, X, C)"
                "(/, A, _, B, C)"[2] = "X" => "(/, A, _, X, C)"
                "<A --> B>"[0] = "a" => "<a --> B>"
                "<A <-> B>"[1] = "X" => "<A <-> X>" // ! 可交换词项解析时重排
                "<A ==> B>"[0] = "a" => "<a ==> B>"
                "<A <=> B>"[1] = "X" => "<A <=> X>" // ! 可交换词项解析时重排
            }
            ok!()
        }

        #[test]
        pub fn reorder_components() -> AResult {
            fn test(mut term: Term, i: usize, new: Term, expected: Term) {
                let mut ref_mut = term.as_compound_mut().unwrap();
                ref_mut.components()[i] = new;
                // * 🚩设置后排序
                ref_mut.reorder_components();
                assert_eq!(term, expected);
            }
            macro_once! {
                macro test($(
                    $term:literal [$i:expr] = $new:literal =>
                    $expected:literal
                )*) {
                    $( test( term!($term), $i, term!($new), term!($expected)); )*
                }
                "{A, B, C}"[1] = "X" => "{A, X, C}" // ! 集合词项在从字符串解析时会重排，但在重排后仍然相等
                "[A, B, C]"[1] = "X" => "[A, X, C]" // ! 集合词项在从字符串解析时会重排，但在重排后仍然相等
                "<A <-> B>"[0] = "a" => "<a <-> B>" // ! 可交换词项解析时重排，但在重排后仍然相等
                "<A <=> B>"[0] = "a" => "<a <=> B>" // ! 可交换词项解析时重排，但在重排后仍然相等
            }
            ok!()
        }
    }

    /// 具所有权的复合词项
    mod compound_term {
        use super::*;
        use std::str::FromStr;

        /// 词项之间的类型转换
        /// * 📄[`Term::try_into`] / [`CompoundTerm::try_from`]
        /// * 📄[`Term::from`] / [`CompoundTerm::into`]
        #[test]
        fn from_into() -> AResult {
            /// 通用测试函数
            fn test(compound: CompoundTerm) {
                // * 🚩首先是一个复合词项
                assert!(compound.is_compound());

                // * 🚩从内部拷贝一个词项后，仍可无损转换为复合词项
                let term: Term = (*compound).clone();
                let _: CompoundTerm = term.try_into().expect("应该是复合词项！");

                // * 🚩解包成普通词项后，仍可无损转换为复合词项
                let term: Term = compound.into();
                let _: CompoundTerm = term.try_into().expect("应该是复合词项！");
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(test_compound!(box $term));
                )*}
                // 普通复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A, B, C)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        #[test]
        fn get_ref() -> AResult {
            /// 通用测试函数
            fn test(compound: CompoundTerm) {
                // * 🚩首先是一个复合词项
                assert!(compound.is_compound());

                // * 🚩获取大小
                let size = compound.get_ref().size();
                println!("{compound}.size() => {size}");

                // * 🚩遍历所有元素
                compound
                    .get_ref()
                    .components()
                    .iter()
                    .enumerate()
                    .for_each(|(i, component)| println!("    [{i}] => {component}"))
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(test_compound!(box $term));
                )*}
                // 普通复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A, B, C)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }

        #[test]
        fn mut_ref() -> AResult {
            /// 通用测试函数
            fn test(mut compound: CompoundTerm) -> AResult {
                // * 🚩首先是一个复合词项
                assert!(compound.is_compound());

                // * 🚩修改：更改第一个元素
                let old_s = compound.to_string();
                let mut mut_ref = compound.mut_ref();
                let first = &mut mut_ref.components()[0];
                let x = term!("X");
                *first = x.clone();
                println!("modification: {old_s:?} => \"{compound}\"");
                assert_eq!(compound.get_ref().components[0], x); // 假定修改后的结果

                // * 🚩遍历修改所有元素
                compound
                    .mut_ref()
                    .components()
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, component)| {
                        *component = Term::from_str(&format!("T{i}")).unwrap()
                    });
                print!(" => \"{compound}\"");

                ok!()
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro test($( $term:literal )*) {$(
                    test(test_compound!(box $term))?;
                )*}
                // 普通复合词项
                "{A}"
                "[A]"
                "(&, A)"
                "(|, A)"
                "(-, A, B)"
                "(~, A, B)"
                "(*, A, B, C)"
                r"(/, R, _)"
                r"(\, R, _)"
                r"(&&, A)"
                r"(||, A)"
                r"(--, A)"
                // 陈述
                "<A --> B>"
                "<A <-> B>"
                "<A ==> B>"
                "<A <=> B>"
            }
            ok!()
        }
    }
}
