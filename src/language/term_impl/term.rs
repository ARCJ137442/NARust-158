//! 📄OpenNARS `nars.language.Term`
//! * ⚠️不包含与特定层数Narsese有关的逻辑
//!   * 📄事关NAL-6的`isConstant`、`renameVariables`方法，不予在此实现
//! * ⚠️不包含与「记忆区」有关的方法
//!   * 📄`make`
//!   * 📝OpenNARS中有关`make`的目的：避免在记忆区中**重复构造**词项
//!     * 🚩已经在概念区中⇒使用已有「概念」的词项
//!     * 📌本质上是「缓存」的需求与作用

use super::*;
use narsese::api::{GetCategory, TermCategory};

/// 📄OpenNARS `nars.language.Term`
impl Term {
    /// 模拟`Term.getName`
    /// * 🆕使用自身内建的「获取名称」方法
    ///   * 相较OpenNARS更**短**
    ///   * 仍能满足OpenNARS的需求
    /// * 🎯OpenNARS原有需求
    ///   * 📌保证「词项不同 ⇔ 名称不同」
    ///   * 📌保证「可用于『概念』『记忆区』的索引」
    ///
    /// # 📄OpenNARS
    ///
    /// Reporting the name of the current Term.
    ///
    /// @return The name of the term as a String
    #[doc(alias = "get_name")]
    pub fn name(&self) -> String {
        self.format_name()
    }

    /// 模拟`Term.getComplexity`
    /// * 🚩逻辑 from OpenNARS
    ///   * 原子 ⇒ 1
    /// //  * 变量 ⇒ 0
    ///   * 复合 ⇒ 1 + 所有组分复杂度之和
    ///
    /// # 📄OpenNARS
    ///
    /// - The syntactic complexity, for constant atomic Term, is 1.
    /// - The complexity of the term is the sum of those of the components plus 1
    /// // - The syntactic complexity of a variable is 0, because it does not refer to * any concept.
    ///
    /// @return The complexity of the term, an integer
    #[doc(alias = "get_complexity")]
    pub fn complexity(&self) -> usize {
        // 剩余类型
        use TermComponents::*;
        match &self.components {
            // 占位符 ⇒ 0
            Empty => 0,
            // 原子/变量 ⇒ 1 | 不包括「变量」
            // * 🚩目前遵照更新的PyNARS设置，将「变量词项」的复杂度定为1
            Word(..) | Variable(..) => 1,
            // 多元 ⇒ 1 + 内部所有词项复杂度之和
            Compound(terms) => 1 + terms.iter().map(Term::complexity).sum::<usize>(),
        }
    }
}

impl GetCategory for Term {
    fn get_category(&self) -> TermCategory {
        use TermCategory::*;
        match self.identifier.as_str() {
            // * 🚩原子：词语、占位符、变量
            WORD | PLACEHOLDER | VAR_INDEPENDENT | VAR_DEPENDENT | VAR_QUERY => Atom,
            // * 🚩陈述：继承、相似、蕴含、等价 | ❌不包括「实例」「属性」「实例属性」
            INHERITANCE_RELATION | IMPLICATION_RELATION | SIMILARITY_RELATION
            | EQUIVALENCE_RELATION => Statement,
            // * 🚩一元：否定
            NEGATION_OPERATOR |
            // * 🚩二元序列：差集
            DIFFERENCE_EXT_OPERATOR | DIFFERENCE_INT_OPERATOR |
            // * 🚩多元序列：乘积、像
            PRODUCT_OPERATOR | IMAGE_EXT_OPERATOR | IMAGE_INT_OPERATOR |
            // * 🚩多元集合：词项集、交集、合取、析取
            SET_EXT_OPERATOR
            | SET_INT_OPERATOR
            | INTERSECTION_EXT_OPERATOR
            | INTERSECTION_INT_OPERATOR
            | CONJUNCTION_OPERATOR
            | DISJUNCTION_OPERATOR => Compound,
            // * 🚩其它⇒panic（不应出现）
            _ => panic!("Unexpected compound term identifier: {}", self.identifier),
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::{asserts, macro_once};

    #[test]
    fn name() -> AResult {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期
            macro fmt($($term:literal => $expected:expr)*) {
                asserts! {$(
                    term!($term).to_string() => $expected
                )*}
            }
            // 占位符
            "_" => "_"
            // 原子词项
            "A" => "A"
            "$A" => "$1" // ! 🚩【2024-06-13 19:02:58】现在对「变量词项」会自动重命名
            "#A" => "#1" // ! 🚩【2024-06-13 19:02:58】现在对「变量词项」会自动重命名
            "?A" => "?1" // ! 🚩【2024-06-13 19:02:58】现在对「变量词项」会自动重命名
            // 复合词项
            "{A, B}" => "{}(A B)"
            "[A, B]" => "[](A B)"
            "(&, A, B)" => "&(A B)"
            "(|, A, B)" => "|(A B)"
            "(-, A, B)" => "(A - B)"
            "(~, A, B)" => "(A ~ B)"
            "(*, A, B)" => "*(A B)"
            r"(/, R, _)" => r"/(R _)"
            r"(\, R, _)" => r"\(R _)"
            r"(/, R, _, A)" => r"/(R _ A)"
            r"(\, R, _, A)" => r"\(R _ A)"
            r"(&&, A, B)" => r"&&(A B)"
            r"(||, A, B)" => r"||(A B)"
            r"(--, A)" => r"(-- A)"
            // 陈述
            "<A --> B>" => "(A --> B)"
            "<A <-> B>" => "(A <-> B)"
            "<A ==> B>" => "(A ==> B)"
            "<A <=> B>" => "(A <=> B)"
            // ! 自动排序
            "<B <-> A>" => "(A <-> B)"
            "<B <=> A>" => "(A <=> B)"
            // ! 变量重命名
            "(*, $e, #d, ?c, $b, #a)" => "*($1 #2 ?3 $4 #5)"
            "(/, $e, #d, ?c, $b, #a, _)" => "/($1 #2 ?3 $4 #5 _)"
        }
        ok!()
    }

    #[test]
    fn complexity() -> AResult {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期
            macro fmt($($term:literal => $expected:expr)*) {
                asserts! {$(
                    term!($term).complexity() => $expected
                )*}
            }
            // 占位符
            "_" => 0
            // 词语
            "A" => 1
            // 变量
            "$A" => 1 // ! 🚩【2024-06-14 00:28:01】现在遵照PyNARS等更新版本的做法
            "#A" => 1
            "?A" => 1
            // 复合词项
            "{A}" => 2
            "[A]" => 2
            "(-, A, B)" => 3
            "(~, A, B)" => 3
            "(&, A, B, C)" => 4
            "(|, A, B, C)" => 4
            "(*, A, B, C, D)" => 5
            r"(/, R, _)" => 2
            r"(\, R, _)" => 2
            r"(/, R, _, A)" => 3
            r"(\, R, _, A)" => 3
            r"(&&, A, B)" => 3
            r"(||, A, B)" => 3
            r"(--, A)" => 2
            r"(--, (--, A))" => 3
            r"(--, (--, (--, A)))" => 4
            // 陈述
            "<A --> B>" => 3
            "<A <-> B>" => 3
            "<A ==> B>" => 3
            "<A <=> B>" => 3
            "<<A --> B> --> B>" => 5
            "<<A <-> B> <-> B>" => 5
            "<<A ==> B> ==> B>" => 5
            "<<A <=> B> <=> B>" => 5
            "<<A --> B> --> <A --> B>>" => 7
            "<<A <-> B> <-> <A <-> B>>" => 7
            "<<A ==> B> ==> <A ==> B>>" => 7
            "<<A <=> B> <=> <A <=> B>>" => 7
        }
        ok!()
    }
}
