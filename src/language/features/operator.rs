//! NAL-8中的「操作符」机制
//! * 📄主要参考自ONA
use crate::language::*;
use crate::symbols::*;
use nar_dev_utils::matches_or;

impl Term {
    /// 用于判断是否为「操作符词项」
    pub fn instanceof_operator(&self) -> bool {
        matches!(self.identifier(), OPERATOR)
    }

    /// 尝试匹配出「操作符」，并返回其中的操作名（若有）
    pub fn as_operator(&self) -> Option<&str> {
        matches_or!(
            ?self.components(),
            TermComponents::Word(name) => name
        )
    }

    /// 🆕检验「是否包含操作符」
    /// * 🚩检查其是否「包含操作符」
    ///   * 自身为「操作符词项」或者其包含「操作符词项」
    #[inline]
    pub fn contain_operator(&self) -> bool {
        self.instanceof_operator() || self.components().contain_operator()
    }
}

impl TermComponents {
    /// 判断「是否包含操作符（词项）」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有操作符」
    /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
    ///   * 📌牺牲一定性能，加快开发速度
    pub fn contain_operator(&self) -> bool {
        self.iter().any(Term::contain_operator)
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::{asserts, macro_once};

    /// 测试/包含操作符
    #[test]
    fn contain_operator() -> AResult {
        macro_once! {
            macro test($($term:expr => $expected:expr)*) {
                asserts! {$(
                    term!($term).contain_operator() => $expected
                )*}
            }
            "<A --> word>"=> false
            "<A --> ^op>"=> true
            "<^op --> A>"=> true
            "<A --> (&&, B, (*, ^op))>"=> true
            "<(&&, B, (*, ^op)) --> A>"=> true
            "<A --> $term>"=> false
            "<A --> #term>"=> false
            "<A --> ?term>"=> false
            "<^Op --> $term>"=> true
            "<^Op --> #term>"=> true
            "<^Op --> ?term>"=> true
        }
        ok!()
    }
}
