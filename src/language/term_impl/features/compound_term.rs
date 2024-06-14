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
        )
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
        self.get_class() == other.get_class()
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
    pub fn as_compound(&self) -> Option<CompoundTermRef> {
        matches_or!(
            ?self.components,
            TermComponents::Compound(ref c) => CompoundTermRef{
                term: self,
                components: c
            }
        )
    }

    /// 🆕尝试将词项作为「复合词项」
    /// * ℹ️[`Self::as_compound`]的可变版本
    pub fn as_compound_mut(&mut self) -> Option<CompoundTermRefMut> {
        matches_or!(
            ?self.components,
            TermComponents::Compound(..) => CompoundTermRefMut {inner   :self}
        )
    }

    /// 🆕尝试将词项作为「复合词项」（未检查）
    /// * 🚩通过判断「内部元素枚举」的类型实现
    ///
    /// # Panics
    ///
    /// ! ⚠️存在「未检查」的风险：在其内部元素不是「复合词项」时，会返回`None`
    pub fn as_compound_unchecked(&self) -> CompoundTermRef {
        match self.components {
            TermComponents::Compound(ref c) => CompoundTermRef {
                term: self,
                components: c,
            },
            _ => unreachable!("未检查：断定的词项不是复合词项"),
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
            // * 🚩二元序列：差集、继承、蕴含 | ❌不包括「实例」「属性」「实例属性」
            DIFFERENCE_EXT_OPERATOR
            | DIFFERENCE_INT_OPERATOR
            | INHERITANCE_RELATION
            | IMPLICATION_RELATION => BinaryVec,
            // * 🚩二元集合：相似、等价
            SIMILARITY_RELATION | EQUIVALENCE_RELATION => BinarySet,
            // * 🚩多元序列：乘积、像
            PRODUCT_OPERATOR | IMAGE_EXT_OPERATOR | IMAGE_INT_OPERATOR => Vec,
            // * 🚩多元集合：词项集、交集、合取、析取
            SET_EXT_OPERATOR
            | SET_INT_OPERATOR
            | INTERSECTION_EXT_OPERATOR
            | INTERSECTION_INT_OPERATOR
            | CONJUNCTION_OPERATOR
            | DISJUNCTION_OPERATOR => Set,
            // * 🚩其它⇒panic（不应出现）
            _ => panic!("Unexpected compound term identifier: {}", self.identifier),
        }
    }
}

/// 🆕作为「复合词项引用」的词项类型
/// * 🎯在程序类型层面表示一个「复合词项」（不可变引用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundTermRef<'a> {
    pub term: &'a Term,
    pub components: &'a [Term],
}

/// 🆕作为「复合词项引用」的词项类型
/// * 🎯在程序类型层面表示一个「复合词项」（可变引用）
/// * ⚠️取舍：因可变引用无法共享，此时需要在构造层面限制
///   * 📌构造时保证「内部组分」为「复合词项」变种
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CompoundTermRefMut<'a> {
    pub inner: &'a mut Term,
}

impl CompoundTermRef<'_> {
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
    pub fn component_at(&self, index: usize) -> Option<&Term> {
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
    ///   * TODO: 需要「复合词项组分」实现`removeAll`浅层移除方法
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
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * 🚩【2024-06-14 10:43:03】遵照改版原意，使用变长数组
    ///   * ℹ️后续需要增删操作
    ///
    /// # 📄OpenNARS
    ///
    /// Clone the component list
    pub fn clone_components(&self) -> Vec<Term> {
        self.components.to_vec()
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
        match self.term.get_class() == other.get_class() {
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
}

impl CompoundTermRefMut<'_> {
    /// 获取内部组分（一定有）
    ///
    /// # Panics
    ///
    /// ! ⚠️若使用了非法的构造方式将「非复合词项」构造入此，则将抛出panic
    pub fn components(&mut self) -> &mut [Term] {
        matches_or!(
            self.inner.components,
            TermComponents::Compound(ref mut components) => components,
            unreachable!("CompoundTermRefMut::components 断言失败：不是复合词项: {}", self.inner)
        )
    }

    /// * 📌可变引用一定能转换成不可变引用
    pub fn as_ref(&self) -> CompoundTermRef {
        self.inner.as_compound_unchecked()
    }

    /* ----- variable-related utilities ----- */

    /// 🆕在变量处理中设置词项
    /// * 🎯变量推理需要使用其方法
    ///
    /// @param &m-this
    /// @param index   []
    /// @param term    []
    pub fn set_term_when_dealing_variables(&mut self, index: usize, term: Term) {
        self.components()[index] = term;
    }

    /// 重命名变量后，更新「是常量」
    pub fn update_after_rename_variables(&mut self) {
        // * 🚩【2024-06-14 13:32:50】↓此句源自OpenNARS
        self.inner.is_constant = true;
        // * ✅无需「重命名」
    }

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

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::{asserts, macro_once};

    macro_rules! compound {
        (mut $($t:tt)*) => {
            term!($($t)*).as_compound_mut().unwrap()
        };
        ($($t:tt)*) => {
            term!($($t)*).as_compound().unwrap()
        };
    }

    /// 复合词项不可变引用
    mod compound_term_ref {
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

        #[test] // TODO: 有待构建
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
        pub fn as_ref() -> AResult {
            macro_once! {
                macro test($($term:literal)*) {
                    asserts! {$(
                            compound!(mut $term).as_ref()
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

        #[test]
        pub fn set_term_when_dealing_variables() -> AResult {
            macro_once! {
                macro test($(
                    $term:literal [$i:expr] = $new:literal =>
                    $expected:literal
                )*) {
                    $(
                        let mut term = term!($term);
                        term.as_compound_mut().unwrap().set_term_when_dealing_variables($i, term!($new));
                        assert_eq!(term, term!($expected));
                    )*
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
        pub fn update_after_rename_variables() -> AResult {
            macro_once! {
                macro test($($term:literal)*) {$(
                    let mut t = term!($term);
                    // * 🚩验证是否会修改`is_constant`
                    t.is_constant = false;
                    t.as_compound_mut().unwrap().update_after_rename_variables();
                    assert!(t.is_constant);
                )*}
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

        #[test]
        pub fn reorder_components() -> AResult {
            macro_once! {
                macro test($(
                    $term:literal [$i:expr] = $new:literal =>
                    $expected:literal
                )*) {
                    $(
                        let mut term = term!($term);
                        let mut ref_mut = term.as_compound_mut().unwrap();
                        ref_mut.set_term_when_dealing_variables($i, term!($new));
                        // * 🚩设置后排序
                        ref_mut.reorder_components();
                        assert_eq!(term, term!($expected));
                    )*
                }
                "{A, B, C}"[1] = "X" => "{A, X, C}" // ! 集合词项在从字符串解析时会重排，但在重排后仍然相等
                "[A, B, C]"[1] = "X" => "[A, X, C]" // ! 集合词项在从字符串解析时会重排，但在重排后仍然相等
                "<A <-> B>"[0] = "a" => "<a <-> B>" // ! 可交换词项解析时重排，但在重排后仍然相等
                "<A <=> B>"[0] = "a" => "<a <=> B>" // ! 可交换词项解析时重排，但在重排后仍然相等
            }
            ok!()
        }
    }
}
