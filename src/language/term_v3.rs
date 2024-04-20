//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * ⚠️最大的分歧：对于其中「无序组分」的表征方式 集合🆚有序数组
//!   * 📄OpenNARS中使用「数组+唯一性筛选+词项有序性」安排「无序不重复组分」
//!   * 📄而Narsese.rs使用「散列集」安排「无序不重复组分」
//!     * 因此「枚举Narsese」的词项不支持「比较」（集合无法比较）
//! * ❌不能使用「内部封装『枚举Narsese词项』再附加『缓存的功能』」
//!   * 📌复合词项内部的「组分词项」并没法定位到其自身
//!   * ❌这意味着：方法不能返回引用类型
//!     * 📌无法实现缓存
//! * ⚠️会引入一些与OpenNARS 1.5.8无关的枚举项
//!   * 💭可能会导致「非必要的『不可达』」
//!   * ❗后续「是否需要发出异常」「如何处理异常」又是一个难点
//!   * ❓是否可能需要实现一个「自己版本的枚举Narsese」
//! * ⚠️因为[`EnumTerm`]没实现「Ord + Display」两个特征，所以还无法自己定义
//!   * 💥但参照OpenNARS，不可能没有这些
//!   * ❌强行实现会违反「孤儿规则」
//! * 🚩【2024-04-20 22:19:04】目前将此方案搁置
//!   * ⇒尝试探索「自行维护一套『枚举Narsese』」的方案
#![allow(unused_imports)]

use narsese::api::GetCategory;
use narsese::conversion::string::impl_enum::format_instances::FORMAT_ASCII;
use narsese::enum_narsese::Term as EnumTerm;
use std::fmt::Display;
use std::hash::Hash;

/// 直接使用「枚举Narsese」的数据结构
/// * 🚩先使用数据结构，然后在此之上附加方法
pub type Term = EnumTerm;

pub trait TraitTerm: Clone + Eq + Hash /* + Ord + Display */ {
    /// 名称
    ///
    /// # 📄OpenNARS
    ///
    /// A Term is identified uniquely by its name, a sequence of characters in a given alphabet (ASCII or Unicode)
    fn name(&self) -> String;

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

impl TraitTerm for Term {
    fn name(&self) -> String {
        FORMAT_ASCII.format(self)
    }

    fn is_variable_term(&self) -> bool {
        use EnumTerm::*;
        matches!(
            self,
            VariableIndependent(_) | VariableDependent(_) | VariableQuery(_)
        )
    }

    fn is_compound_term(&self) -> bool {
        self.is_compound()
    }

    fn rename_variables(&mut self) {
        todo!("不同词项有不同的实现")
    }
}
