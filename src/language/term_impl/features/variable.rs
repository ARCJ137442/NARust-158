//! 📄OpenNARS `nars.language.Variable`
//! * 📌与NAL-6有关的「变量」逻辑
//!   * 📄`isConstant`
//! * 🚩【2024-06-14 17:31:44】只包含最基本的「变量存在性判定」「是否为常量」等基本逻辑
//!   * ⚠️涉及「变量统一」「变量重命名」等逻辑，放置在专用的「变量推理」代码中
//!
//! # 方法列表
//! 🕒最后更新：【2024-06-19 02:05:25】
//!
//! * `isConstant`
//! * `getType` => `getVariableType`
//! * `containVarI`
//! * `containVarD`
//! * `containVarQ`
//! * `containVar`
//!
//! # 📄OpenNARS
//!
//! A variable term, which does not correspond to a concept

use crate::language::*;
use crate::symbols::*;
use nar_dev_utils::matches_or;

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

    /// 🆕用于判断「是否为独立变量」
    pub fn instanceof_variable_i(&self) -> bool {
        self.identifier == VAR_INDEPENDENT
    }

    /// 🆕用于判断「是否为非独变量」
    pub fn instanceof_variable_d(&self) -> bool {
        self.identifier == VAR_DEPENDENT
    }

    /// 🆕用于判断「是否为查询变量」
    pub fn instanceof_variable_q(&self) -> bool {
        self.identifier == VAR_QUERY
    }

    /// 尝试匹配出「变量」，并返回其中的编号（若有）
    pub fn as_variable(&self) -> Option<usize> {
        matches_or!(
            ?self.components,
            TermComponents::Variable(n) => n
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
    /// * ✅【2024-06-19 02:06:12】跟随最新改版更新，删去字段并铺开实现此功能
    /// * ♻️【2024-06-26 02:07:27】重构修正：禁止「占位符」作为「常量词项」
    /// * ♻️【2024-07-31 21:41:49】修正：不再将查询变量计入「常量词项」
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
        !self.instanceof_variable() && !self.is_placeholder() && !self.contains_sole_variable()
    }

    /// 🆕检查自身是否包含有「孤立非查询变量」
    /// * 📄复刻自OpenNARS改版逻辑
    fn contains_sole_variable(&self) -> bool {
        use std::collections::HashMap;

        /// * 🚩计算「非查询变量数目集」
        fn variable_count_map(this: &Term) -> HashMap<usize, usize> {
            let mut var_count_map = HashMap::new();
            this.for_each_atom(&mut |atom| {
                if let Some(n) = atom.as_variable() {
                    // * 🚩非查询变量
                    if !atom.instanceof_variable_q() {
                        let new_value = match var_count_map.get(&n) {
                            Some(count) => count + 1,
                            None => 1,
                        };
                        var_count_map.insert(n, new_value);
                    }
                }
            });
            var_count_map
        }

        // * 🚩计算并过滤
        let var_count_map = variable_count_map(self);
        var_count_map.values().any(|&count| count < 2)
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

    /// 🆕获取多个词项中编号最大的变量词项id
    pub fn maximum_variable_id_multi<'s>(terms: impl IntoIterator<Item = &'s Term>) -> usize {
        terms
            .into_iter()
            .map(Term::maximum_variable_id) // 统计各个词项的最大变量id
            .max() // 取最大值
            .unwrap_or(0) // 以0为补充（即便空集）
    }
}

/// 🆕获取编号最大的变量词项id
/// * 🎯兼容「词项」与「词项数组」
pub trait MaximumVariableId {
    fn maximum_variable_id(&self) -> usize;
}

/// 词项本身
impl MaximumVariableId for Term {
    /// 🆕获取一个词项中编号最大的变量词项id
    fn maximum_variable_id(&self) -> usize {
        use TermComponents::*;
        match self.components() {
            // 变量⇒自身id
            Variable(id) => *id,
            // 内含词项⇒递归深入
            Compound(terms) => Term::maximum_variable_id_multi(terms.iter()),
            // 其它⇒0 | 后续开放补充
            Empty | Word(..) => 0,
        }
    }
}

/// 词项引用
impl MaximumVariableId for &Term {
    fn maximum_variable_id(&self) -> usize {
        Term::maximum_variable_id(*self)
    }
}

/// 兼容词项数组
impl<const N: usize> MaximumVariableId for [Term; N] {
    fn maximum_variable_id(&self) -> usize {
        Term::maximum_variable_id_multi(self)
    }
}

/// 兼容引用数组
impl<const N: usize> MaximumVariableId for [&Term; N] {
    fn maximum_variable_id(&self) -> usize {
        // * 🚩使用`cloned`将`&&Term`转换为`&Term`
        Term::maximum_variable_id_multi(self.iter().cloned())
    }
}

/// 兼容数组切片
impl MaximumVariableId for [Term] {
    fn maximum_variable_id(&self) -> usize {
        Term::maximum_variable_id_multi(self)
    }
}

/// 兼容引用数组切片
impl MaximumVariableId for [&Term] {
    fn maximum_variable_id(&self) -> usize {
        // * 🚩使用`cloned`将`&&Term`转换为`&Term`
        Term::maximum_variable_id_multi(self.iter().cloned())
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
    use crate::{ok, util::AResult};
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
            // * 📌【2024-06-19 02:27:06】现在改版中成功的项：
            // 查询变量
            "<A --> ?var_word>" => true
            "<?this --> ?that>" => true
            // 封闭词项
            "<<A --> $1> ==> <B --> $1>>" => true
            "<<$2 --> $1> ==> <$1 --> $2>>" => true
            "(*, $1, $1)" => true
        }
        ok!()
    }
}
