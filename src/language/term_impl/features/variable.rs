//! 📄OpenNARS `nars.language.Variable`
//! * 📌与NAL-6有关的「变量」逻辑
//!   * 📄`isConstant`
//! * 🚩【2024-06-14 17:31:44】只包含最基本的「变量存在性判定」「是否为常量」等基本逻辑
//!   * ⚠️涉及「变量统一」「变量重命名」等逻辑，放置在专用的「变量推理」代码中
//!
//! # 方法列表
//! 🕒最后更新：【2024-04-24 14:32:52】
//!
//! * `isConstant`
//! * `renameVariables`
//! * `applySubstitute`
//! * `getType` => `getVariableType`
//! * `containVarI`
//! * `containVarD`
//! * `containVarQ`
//! * `containVar`
//!
//! # 📄OpenNARS
//!
//! A variable term, which does not correspond to a concept

use term_impl::features::compound_term::CompoundTermRef;

use crate::io::symbols::*;
use crate::language::*;
use std::collections::HashMap;

impl Term {
    /// 用于判断是否为「变量词项」
    /// * 📄OpenNARS `instanceof Variable` 逻辑
    /// * 🎯判断「[是否内含变量](Self::contain_var)」
    pub fn instanceof_variable(&self) -> bool {
        matches!(
            self.identifier.as_str(),
            VAR_INDEPENDENT | VAR_DEPENDENT | VAR_QUERY
        )
    }

    /// 📄OpenNARS `Term.isConstant` 属性
    /// * 🚩检查其是否为「常量」：自身是否「不含变量」
    /// * 🎯决定其是否能**成为**一个「概念」（被作为「概念」存入记忆区）
    /// * ❓OpenNARS中在「构造语句」时又会将`isConstant`属性置为`true`，这是为何
    ///   * 📝被`Sentence(..)`调用的`CompoundTerm.renameVariables()`会直接将词项「视作常量」
    ///   * 💭这似乎是被认为「即便全是变量，只要是【被作为语句输入过】的，就会被认作是『常量』」
    ///   * 📝然后这个「是否常量」会在「记忆区」中被认作「是否能从中获取概念」的依据：`if (!term.isConstant()) { return null; }`
    /// * 🚩【2024-04-21 23:46:12】现在变为「只读属性」：接受OpenNARS中有关「设置语句时/替换变量后 变为『常量』」的设定
    ///   * 💫【2024-04-22 00:03:10】后续仍然有一堆复杂逻辑要考虑
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether the current Term can name a Concept.
    ///
    /// - A Term is constant by default
    /// - A variable is not constant
    /// - (for `CompoundTerm`) check if the term contains free variable
    #[inline(always)]
    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    /// 📄OpenNARS `Variable.containVar` 方法
    /// * 🚩检查其是否「包含变量」
    ///   * 自身为「变量词项」或者其包含「变量词项」
    /// * 🎯用于决定复合词项是否为「常量」
    /// * 📝OpenNARS中对于复合词项的`isConstant`属性采用「惰性获取」的机制
    ///   * `isConstant`作为`!Variable.containVar(name)`进行初始化
    /// * 🆕实现方法：不同于OpenNARS「直接从字符串中搜索子串」的方式，基于递归方法设计
    ///
    /// # 📄OpenNARS
    ///
    /// Check whether a string represent a name of a term that contains a variable
    #[inline]
    pub fn contain_var(&self) -> bool {
        self.instanceof_variable() || self.components.contain_var()
    }

    /// 📄OpenNARS `Variable.containVarI` 方法
    /// * 🎯判断「是否包含指定类型的变量」
    /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
    pub fn contain_var_i(&self) -> bool {
        self.contain_type(VAR_INDEPENDENT)
    }

    /// 📄OpenNARS `Variable.containVarD` 方法
    /// * 🎯判断「是否包含指定类型的变量」
    /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
    pub fn contain_var_d(&self) -> bool {
        self.contain_type(VAR_DEPENDENT)
    }

    /// 📄OpenNARS `Variable.containVarQ` 方法
    /// * 🎯判断「是否包含指定类型的变量」
    /// * 🚩通过「判断是否包含指定标识符的词项」完成判断
    pub fn contain_var_q(&self) -> bool {
        self.contain_type(VAR_QUERY)
    }

    /// 📄OpenNARS `Variable.getType` 方法
    /// * 🎯在OpenNARS中仅用于「判断变量类型相等」
    /// * 🚩归并到「判断词项标识符相等」
    ///
    /// # 📄OpenNARS
    ///
    /// Get the type of the variable
    #[inline(always)]
    pub fn get_variable_type(&self) -> &str {
        &self.identifier
    }
}

impl TermComponents {
    /// 判断「是否包含变量（词项）」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
    ///   * 📌牺牲一定性能，加快开发速度
    pub fn contain_var(&self) -> bool {
        self.iter().any(Term::contain_var)
    }
}
/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::{asserts, macro_once};

    /// 测试/包含变量
    /// * ✨同时包含对「是否常量」的测试
    #[test]
    fn contain_var() -> AResult {
        macro_once! {
            macro test($($term:expr => $expected:expr)*) {
                asserts! {$(
                    term!($term).contain_var() => $expected
                )*}
            }
            "<A --> var_word>"=> false
            "<A --> $var_word>"=> true
            "<A --> #var_word>"=> true
            "<A --> ?var_word>"=> true
        }
        ok!()
    }

    #[test]
    fn is_constant() -> AResult {
        macro_once! {
            macro test($($term:expr => $expected:expr)*) {
                asserts! {$(
                    term!($term).is_constant() => $expected
                )*}
            }
            "<A --> var_word>" => true
            "<A --> $var_word>" => false
            "<A --> #var_word>" => false
            "<A --> ?var_word>" => false
            "<<A --> $1> ==> <B --> $1>>" => false
            // ! ↑参考自OpenNARS：最初是false，但在「作为语句输入」后，转变为true
        }
        ok!()
    }
}
