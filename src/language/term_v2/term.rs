//! 📄OpenNARS `nars.language.Term`
//! * ⚠️不包含与特定层数Narsese有关的逻辑
//!   * 📄事关NAL-6的`isConstant`、`renameVariables`方法，不予在此实现
//! * ⚠️不包含与「记忆区」有关的方法
//!   * 📄`make`
//!   * 📝OpenNARS中有关`make`的目的：避免在记忆区中**重复构造**词项
//!     * 🚩已经在概念区中⇒使用已有「概念」的词项
//!     * 📌本质上是「缓存」的需求与作用

use super::*;
use nar_dev_utils::if_return;

/// 📄OpenNARS `nars.language.Term`
impl Term {
    /// 📄OpenNARS `Term.getName` 方法
    /// * 🆕使用自身内建的「获取名称」方法
    ///   * 相较OpenNARS更**短**
    ///   * 仍能满足OpenNARS的需求
    /// * 🎯OpenNARS原有需求
    ///   * 📌保证「词项不同 ⇔ 名称不同」
    ///   * 📌保证「可用于『概念』『记忆区』的索引」
    pub fn get_name(&self) -> String {
        self.format_name()
    }

    /// 📄OpenNARS `Term.getComplexity` 方法
    /// * 🚩逻辑 from OpenNARS
    ///   * 词语 ⇒ 1
    ///   * 变量 ⇒ 0
    ///   * 复合 ⇒ 1 + 所有组分复杂度之和
    ///
    /// # 📄OpenNARS
    ///
    /// - The syntactic complexity, for constant atomic Term, is 1.
    /// - The complexity of the term is the sum of those of the components plus 1
    /// - The syntactic complexity of a variable is 0, because it does not refer to * any concept.
    ///
    /// @return The complexity of the term, an integer
    pub fn get_complexity(&self) -> usize {
        // 对「变量」特殊处理：不引用到任何「概念」
        if_return! {
            self.instanceof_variable() => 0
        }
        // 剩余类型
        use TermComponents::*;
        match &*self.components {
            // 占位符 ⇒ 0
            Empty => 0,
            // 原子 ⇒ 1 | 不包括「变量」
            Named(..) => 1,
            // 一元 ⇒ 1 + 内部词项复杂度
            Unary(term) => 1 + term.get_complexity(),
            // 二元 ⇒ 1 + 内部所有词项复杂度之和
            Binary(term1, term2) => 1 + term1.get_complexity() + term2.get_complexity(),
            // 多元 ⇒ 1 + 内部所有词项复杂度之和
            Multi(terms) | MultiIndexed(_, terms) => {
                1 + terms.iter().map(Term::get_complexity).sum::<usize>()
            }
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use anyhow::Result;
    use nar_dev_utils::{asserts, macro_once};

    #[test]
    fn get_name() -> Result<()> {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期
            macro fmt($($term:literal => $expected:expr)*) {
                asserts! {$(
                    format!("{}", term!($term)) => $expected
                )*}
            }
            // 占位符
            "_" => "_"
            // 原子词项
            "A" => "A"
            "$A" => "$A"
            "#A" => "#A"
            "?A" => "?A"
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
        }
        Ok(())
    }

    #[test]
    fn get_complexity() -> Result<()> {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期
            macro fmt($($term:literal => $expected:expr)*) {
                asserts! {$(
                    term!($term).get_complexity() => $expected
                )*}
            }
            // 占位符
            "_" => 0
            // 词语
            "A" => 1
            // 变量
            "$A" => 0
            "#A" => 0
            "?A" => 0
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
        Ok(())
    }
}
