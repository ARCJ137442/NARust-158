//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * 🚩【2024-04-20 21:02:31】使用「抽象特征+特征对象+强制转换」的思路，疑虑重重
//!   * ❌强制转换中[`Any`]要求`'static`生存期——明显不能接受
//!   * ❓基于「特征-实现」的代码多态，需要「一种特征对象→另一种特征对象」的转换——几乎不可能
//! * 🚩【2024-04-20 21:11:42】目前将此方案搁置
//!   * ⇒尝试探索「基于统一struct」的方案

use std::{fmt::Display, hash::Hash};

/// 作为「抽象类」的词项
/// * 🚩【2024-04-20 20:30:41】目前让「词项」和「原子词项」分立
///   * 基于抽象特征定义「词项」，后续转换则使用**特征对象**
/// * 📄其「所含特征」与OpenNARS中`clone`、`equals`、`hashCode`对应
///
/// # 📄OpenNARS
///
/// Term is the basic component of Narsese, and the object of processing in NARS.
/// A Term may have an associated Concept containing relations with other Terms.
/// It is not linked in the Term, because a Concept may be forgot while the Term exists. Multiple objects may represent the same Term.
///
/// ## 作为特征的「实现」
///
/// ### Cloneable => [`Clone`]
///
/// Make a new Term with the same name.
///
/// ### equals => [`Eq`]
///
/// Equal terms have identical name, though not necessarily the same reference.
///
/// ### hashCode => [`Hash`]
///
/// Produce a hash code for the term
///
/// ### compareTo => [`Ord`]
///
/// Orders among terms: variable < atomic < compound
///
/// ### toString => [`Display`]
///
/// The same as getName by default, used in display only.
///
/// @return The name of the term as a String
pub trait Term: Clone + Eq + Hash + Ord + Display {
    /// 名称
    ///
    /// # 📄OpenNARS
    ///
    /// A Term is identified uniquely by its name, a sequence of characters in a given alphabet (ASCII or Unicode)
    fn name(&self) -> &str;

    /// 是否为「常量」
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the current Term can name a Concept.
    ///
    /// @return A Term is constant by default
    #[inline]
    fn is_constant(&self) -> bool {
        true
    }

    /// 🆕是否为「变量词项」
    /// * 🎯用于「偏序」关系的判断：变量 < 原子 < 复合
    /// * 🎯用于模拟OpenNARS中`instanceof Variable`
    ///   * 💭【2024-04-20 20:43:21】所有`instanceof`在Rust中都要被大幅度重写
    /// * ⚠️与「是否为常量」不同：一个复合词项，既有可能是「常量」亦有可能非「常量」
    ///   * 📄如：`<$1 --> A>`
    fn is_variable_term(&self) -> bool;

    /// 🆕是否为「复合词项」
    /// * 🎯用于「偏序」关系的判断：变量 < 原子 < 复合
    /// * 🎯用于模拟OpenNARS中`instanceof CompoundTerm`
    fn is_compound_term(&self) -> bool;

    /// 重命名变量
    ///
    /// # 📄OpenNARS
    ///
    /// Blank method to be override in CompoundTerm
    fn rename_variables(&mut self);

    /// 词项复杂度
    ///
    /// # 📄OpenNARS
    ///
    /// The syntactic complexity, for constant atomic Term, is 1.
    ///
    /// @return The complexity of the term, an integer
    #[inline]
    fn get_complexity(&self) -> usize {
        1
    }

    // ⚠️不提供原OpenNARS`toString`函数，改为要求`Display`特征
    // * ❌「批量实现[`Display`]」违反「孤儿规则」
}
