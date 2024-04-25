//! 📄OpenNARS `nars.language.CompoundTerm`
//! * ⚠️不包含与NAL-6有关的「变量」逻辑
//!   * 📄`isConstant`、`renameVariables`
//! * ⚠️不包含与「记忆区」有关的方法
//!   * 📄`addComponents`、`reduceComponents`
//!
//! # 方法列表
//! 🕒最后更新：【2024-04-21 17:10:46】
//!
//! * `isCommutative`
//! * `size`
//! * `componentAt`
//! * `componentAt`
//! * `getComponents`
//! * `cloneComponents`
//! * `containComponent`
//! * `containTerm`
//! * `containAllComponents`
//!
//! # 📄OpenNARS
//!
//! A CompoundTerm is a Term with internal (syntactic) structure
//!
//! A CompoundTerm consists of a term operator with one or more component Terms.
//!
//! This abstract class contains default methods for all CompoundTerms.

use super::*;
impl Term {
    /// 用于判断是否为「复合词项」
    /// * ⚠️包括陈述
    /// * 📄OpenNARS `instanceof CompoundTerm` 逻辑
    pub fn instanceof_compound(&self) -> bool {
        self.instanceof_statement()
            || matches!(
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

    /// 📄OpenNARS `CompoundTerm.isCommutative` 属性
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

    /// 📄OpenNARS `CompoundTerm.size` 属性
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

    /// 📄OpenNARS `CompoundTerm.componentAt` 方法
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

    /// 📄OpenNARS `CompoundTerm.componentAt` 方法
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

    /// 📄OpenNARS `CompoundTerm.getComponents` 属性
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * 🚩【2024-04-21 16:11:59】目前只需不可变引用
    ///   * 🔎OpenNARS中大部分用法是「只读」情形
    ///
    /// # 📄OpenNARS
    ///
    /// Get the component list
    #[inline]
    pub fn get_components(&self) -> impl Iterator<Item = &Term> {
        self.components.iter()
    }

    /// 📄OpenNARS `CompoundTerm.cloneComponents` 方法
    /// * 🚩直接连接到[`TermComponents`]的方法
    /// * ✅直接使用自动派生的[`TermComponents::clone`]方法，且不需要OpenNARS中的`cloneList`
    ///
    /// # 📄OpenNARS
    ///
    /// Clone the component list
    pub fn clone_components(&self) -> TermComponents {
        *self.components.clone()
    }

    /// 📄OpenNARS `CompoundTerm.containComponent` 方法
    /// * 🎯检查其是否包含**直接**组分
    /// * 🚩直接基于已有迭代器方法
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the compound contains a certain component
    pub fn contain_component(&self, component: &Term) -> bool {
        self.get_components().any(|term| term == component)
    }

    /// 📄OpenNARS `CompoundTerm.containTerm` 方法
    /// * 🎯检查其是否**递归**包含组分
    /// * 🚩直接基于已有迭代器方法
    ///
    /// # 📄OpenNARS
    ///
    /// Recursively check if a compound contains a term
    #[allow(clippy::only_used_in_recursion)]
    pub fn contain_term(&self, term: &Term) -> bool {
        self.get_components()
            .any(|component| component.contain_term(term))
    }

    /// 🆕用于替代Java的`getClass`
    #[inline(always)]
    pub fn get_class(&self) -> &str {
        &self.identifier
    }

    /// 📄OpenNARS `CompoundTerm.containAllComponents` 方法
    /// * 🎯分情况检查「是否包含所有组分」
    ///   * 📌同类⇒检查其是否包含`other`的所有组分
    ///   * 📌异类⇒检查其是否包含`other`作为整体
    /// * 🚩直接基于已有迭代器方法
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the compound contains all components of another term, or that term as a whole
    pub fn contain_all_components(&self, other: &Term) -> bool {
        match self.get_class() == other.get_class() {
            true => other
                .get_components()
                .all(|should_in| self.contain_component(should_in)),
            false => self.contain_component(other),
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    // TODO: 添加测试内容
}
