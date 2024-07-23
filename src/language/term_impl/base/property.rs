//! 实现 / 属性（内建）
//! * 🎯非OpenNARS所定义之「属性」「方法」
//!   * 📌至少并非OpenNARS原先所定义的

use super::structs::*;
use crate::io::symbols::*;
use crate::util::ToDisplayAndBrief;
use narsese::{
    conversion::string::impl_lexical::format_instances::FORMAT_ASCII, lexical::Term as TermLexical,
};

/// 内建属性
impl Term {
    /// 只读的「标识符」属性
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// 只读的「组分」属性
    pub fn components(&self) -> &TermComponents {
        &self.components
    }

    /// language内部可写的「组分」属性
    pub(in crate::language) fn components_mut(&mut self) -> &mut TermComponents {
        &mut self.components
    }

    /// 判断其是否为「占位符」
    /// * 🎯【2024-04-21 01:04:17】在「词法折叠」中首次使用
    pub fn is_placeholder(&self) -> bool {
        self.identifier == PLACEHOLDER
    }

    /// 快捷获取「标识符-组分」二元组
    /// * 🎯用于很多地方的「类型匹配」
    pub fn id_comp(&self) -> (&str, &TermComponents) {
        (&self.identifier, &self.components)
    }

    /// 快捷获取「标识符-组分」二元组，并提供可变机会
    /// * 🚩【2024-04-21 00:59:20】现在正常返回其两重可变引用
    /// * 📝【2024-04-21 00:58:58】当「标识符」为「静态字串」时，不能对其内部的`&str`属性进行修改
    ///   * 📌使用`&mut &str`会遇到生命周期问题
    ///   * 📌实际上「修改类型」本身亦不常用
    pub fn id_comp_mut(&mut self) -> (&mut str, &mut TermComponents) {
        (&mut self.identifier, &mut self.components)
    }

    /// 判断「是否包含指定类型的词项」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    pub fn contain_type(&self, identifier: &str) -> bool {
        self.identifier == identifier || self.components.contain_type(identifier)
    }

    /// 遍历其中所有原子词项
    /// * 🎯找到其中所有的变量
    /// * ⚠️外延像/内涵像 中的占位符
    /// * ⚠️需要传入闭包的可变引用，而非闭包本身
    ///   * 📌中间「递归深入」需要重复调用（传入）闭包
    /// * 📄词语、变量
    /// * 📄占位符
    pub fn for_each_atom(&self, f: &mut impl FnMut(&Term)) {
        use TermComponents::*;
        match self.components() {
            // 无组分⇒遍历自身
            Empty | Word(..) | Variable(..) | Interval(..) => f(self),
            // 内含词项⇒递归深入
            Compound(terms) => {
                for term in terms.iter() {
                    term.for_each_atom(f);
                }
            }
        }
    }

    /// 遍历其中所有原子词项（可变版本）
    /// * [`Self::for_each_atom`]的可变版本
    /// * 📌仅在整个库内部使用
    pub(crate) fn for_each_atom_mut(&mut self, f: &mut impl FnMut(&Term)) {
        use TermComponents::*;
        match self.components_mut() {
            // 无组分⇒遍历自身
            Empty | Word(..) | Variable(..) | Interval(..) => f(self),
            // 内含词项⇒递归深入
            Compound(terms) => {
                for term in terms.iter_mut() {
                    term.for_each_atom_mut(f);
                }
            }
        }
    }
}

/// 实现[`Display`]
/// * 🎯调试时便于展现内部结构
/// * ⚡性能友好
/// * ⚠️并非CommonNarsese语法
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format_name())
    }
}

/// 自动实现[`ToDisplayAndBrief`]
/// * 🚩【2024-05-08 23:30:59】「简略显示」与「完全显示」相同
/// * 🚩【2024-05-08 23:31:32】目前使用ASCII格式化器去做，性能可能会低
impl ToDisplayAndBrief for Term {
    fn to_display(&self) -> String {
        FORMAT_ASCII.format(&TermLexical::from(self))
    }
}

/// 内建属性
impl TermComponents {
    /// 获取「组分」的大小
    /// * ⚠️对于「带索引序列」不包括「索引」
    ///   * 📄对「像」不包括「像占位符」
    pub fn len(&self) -> usize {
        use TermComponents::*;
        match self {
            // 无组分
            Empty | Word(..) | Variable(..) | Interval(..) => 0,
            // 不定数目
            Compound(terms) => terms.len(),
        }
    }

    /// 获取「组分是否为空」
    /// * 🎯自clippy提示而设
    pub fn is_empty(&self) -> bool {
        use TermComponents::*;
        match self {
            // 一定空
            Empty | Word(..) | Variable(..) | Interval(..) => true,
            // 可能空
            Compound(terms) => terms.is_empty(),
        }
    }

    /// 获取指定位置的组分（不一定有）
    /// * ⚠️对于「带索引序列」不受「索引」影响
    ///   * 📄对「像」不受「像占位符」影响
    pub fn get(&self, index: usize) -> Option<&Term> {
        use TermComponents::*;
        match self {
            // 无组分
            Empty | Word(..) | Variable(..) | Interval(..) => None,
            // 有组分
            Compound(terms) => terms.get(index),
        }
    }

    /// 获取指定位置的组分（不检查，直接返回元素）
    /// * ⚠️对于「带索引序列」不受「索引」影响
    ///   * 📄对「像」不受「像占位符」影响
    ///
    /// # Safety
    ///
    /// ⚠️只有在「确保索引不会越界」才不会引发panic和未定义行为（UB）
    pub unsafe fn get_unchecked(&self, index: usize) -> &Term {
        use TermComponents::*;
        match self {
            // 有组分
            Compound(terms) => terms.get_unchecked(index),
            // 其它情况⇒panic
            _ => panic!("尝试在非法位置 {index} 获取词项：{self:?}"),
        }
    }

    /// 获取其中「所有元素」的迭代器
    /// * 🚩返回一个迭代器，迭代其中所有「元素」
    /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
    pub fn iter(&self) -> impl Iterator<Item = &Term> {
        use TermComponents::*;
        // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
        let b: Box<dyn Iterator<Item = &Term>> = match self {
            // 一定空
            Empty | Word(..) | Variable(..) | Interval(..) => Box::new(None.into_iter()),
            // 可能空
            Compound(terms) => Box::new(terms.iter()),
        };
        b
    }

    /// （作为无序不重复集合）排序内部词项并去重
    /// * 🎯表征「可交换词项（无序不重复词项）」的「构造时整理」与「修改后整理」
    /// * 🎯提供统一的方法，整理内部词项而不依赖外界
    /// * 🎯用作「集合中替换元素后，重新排序（并去重）」
    ///   * ⚠️不会在「固定数目词项」中去重
    ///   * 📄NAL-6「变量替换」
    /// * ⚠️暂且封闭：不让外界随意调用 破坏其内部结构
    /// * ⚠️只会排序内部的一层词项
    pub(crate) fn sort_dedup(self) -> Self {
        use TermComponents::*;
        match self {
            // 无组分 ⇒ 不排序
            Empty | Word(..) | Variable(..) | Interval(..) => self,
            // 不定数目⇒直接对数组重排并去重
            Compound(terms) => Self::Compound(Self::sort_dedup_terms(terms)),
        }
    }

    /// 在不可变长数组中对数组进行排序并去重
    pub fn sort_dedup_terms(terms: Box<[Term]>) -> Box<[Term]> {
        // 转换成变长数组
        let mut new_terms = Vec::from(terms);
        // * 重排+去重
        Self::sort_dedup_term_vec(&mut new_terms);
        // 转换回定长数组
        new_terms.into_boxed_slice()
    }

    /// 对「词项数组」重排并去重
    pub fn sort_dedup_term_vec(terms: &mut Vec<Term>) {
        // 重排 | ✅保证去重不改变顺序
        terms.sort();
        // 去重 | ⚠️危险：会改变词项长度
        terms.dedup();
    }

    /// 获取内部所有词项，拷贝成变长数组
    /// * 🎯用于复合词项增删相关
    pub fn clone_to_vec(&self) -> Vec<Term> {
        use TermComponents::*;
        match self {
            // * 🚩原子词项⇒空数组
            Empty | Word(..) | Variable(..) | Interval(..) => vec![],
            // * 🚩复合词项⇒使用`to_vec`拷贝数组
            Compound(terms) => terms.to_vec(),
        }
    }

    /// 判断「是否包含指定类型的词项」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
    ///   * 📌牺牲一定性能，加快开发速度
    pub fn contain_type(&self, identifier: &str) -> bool {
        self.iter().any(|term| term.contain_type(identifier))
    }

    /// 判断「结构模式上是否匹配」
    /// * 🚩判断二者在「结构大小」与（可能有的）「结构索引」是否符合
    /// * ⚠️非递归：不会递归比较「组分是否对应匹配」
    /// * 🎯变量替换中的「相同结构之模式替换」
    /// * 📄`variable::find_substitute`
    pub fn structural_match(&self, other: &Self) -> bool {
        use TermComponents::*;
        match (self, other) {
            // 同类型 / 空 | 同类型 / 具名 | 同类型 / 变量
            (Empty, Empty) | (Word(..), Word(..)) | (Variable(..), Variable(..)) => true,
            // 同类型 / 多元
            (Compound(terms1), Compound(terms2)) => terms1.len() == terms2.len(),
            // 其它情形（类型相异）
            _ => false,
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::asserts;

    /// 测试 / [`Term`]
    mod term {
        use super::*;
        use nar_dev_utils::macro_once;

        #[test]
        fn eq() -> AResult {
            macro_once! {
                // * 🚩模式：左边词项 运算符 右边字符串
                macro eq($( $left:literal $op:tt $right:expr )*) {
                    asserts! {$(
                        term!($left) $op term!($right),
                    )*}
                }
                // 二次构造
                "A" == "A"
                "<A --> B>" == "<A-->B>"
                "[A]" == "[A]"
                // 可交换性
                "<A <-> B>" == "<B <-> A>"
                "(&, C, A, B)" == "(&, B, C, A)"
                "{C, A, B}" == "{B, C, A}"
                // 自动转换
                r"(/, _, A, B)" == "(*, A, B)"
                r"(\, _, A, B)" == "(*, A, B)"
                // 不等 / 标识符
                "$A" != "A"
                "$A" != "#A"
                r"(\, A, _, B)" != r"(/, A, _, B)"
                "<A <-> B>" != "<A <=> B>"
                // 不等 / 元素
                "A" != "a"
                "(*, A, B, C)" != "(*, A, B)"
                "(*, A, B, C)" != "(*, A, B, c)"
                "(/, A, B, _)" != "(/, A, _, B)"
                "{C, A, B}" != "{B, C}"
            }
            ok!()
        }

        /// 测试 / 散列
        /// * 🚩【2024-04-25 09:24:58】仅测试其「可散列化」
        #[test]
        fn hash() -> AResult {
            use std::collections::{HashMap, HashSet};
            use std::hash::RandomState;
            // 创建
            let mut map = HashMap::from([(term!("A"), term!("B")), (term!("C"), term!("D"))]);
            let mut set: HashSet<Term, RandomState> = HashSet::from_iter(map.keys().cloned());
            asserts! {
                map.get(&term!("A")) => Some(&term!("B")),
                map.get(&term!("C")) => Some(&term!("D")),
                map.get(&term!("E")) => None,
                set.contains(&term!("A"))
                set.contains(&term!("C"))
            }
            // 修改
            map.insert(term!("D"), term!("C"));
            for v in map.values() {
                set.insert(v.clone());
            }
            asserts! {
                map.get(&term!("D")) => Some(&term!("C")),
                set.contains(&term!("B"))
                set.contains(&term!("D"))
            }
            // 结束
            dbg!(&map, &set);
            ok!()
        }

        #[test]
        fn identifier() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro identifier($( $s:literal => $expected:expr )*) {
                    asserts! {$(
                        term!($s).identifier() => $expected,
                    )*}
                }
                // 占位符
                "_" => PLACEHOLDER
                // 原子词项
                "A" => WORD
                "$A" => VAR_INDEPENDENT
                "#A" => VAR_DEPENDENT
                "?A" => VAR_QUERY
                // 复合词项
                "{A}" => SET_EXT_OPERATOR
                "[A]" => SET_INT_OPERATOR
                "(&, A)" => INTERSECTION_EXT_OPERATOR
                "(|, A)" => INTERSECTION_INT_OPERATOR
                "(-, A, B)" => DIFFERENCE_EXT_OPERATOR
                "(~, A, B)" => DIFFERENCE_INT_OPERATOR
                "(*, A)" => PRODUCT_OPERATOR
                r"(/, R, _)" => IMAGE_EXT_OPERATOR
                r"(\, R, _)" => IMAGE_INT_OPERATOR
                r"(&&, A)" => CONJUNCTION_OPERATOR
                r"(||, A)" => DISJUNCTION_OPERATOR
                r"(--, A)" => NEGATION_OPERATOR
                // 陈述
                "<A --> B>" => INHERITANCE_RELATION
                "<A <-> B>" => SIMILARITY_RELATION
                "<A ==> B>" => IMPLICATION_RELATION
                "<A <=> B>" => EQUIVALENCE_RELATION
            }
            ok!()
        }

        #[test]
        fn components() -> AResult {
            use TermComponents::*;
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期模式
                macro components($( $s:literal => $expected:pat )*) {
                    asserts! {$(
                        term!($s).components() => @$expected,
                    )*}
                }
                // 空（一般不会在外部出现）
                "_" => Empty
                // 具名
                "A" => Word(..)
                // 变量
                "$A" => Variable(..)
                "#A" => Variable(..)
                "?A" => Variable(..)
                // 一元
                "(--, A)" => Compound(..)
                // 二元
                "(-, A, B)" => Compound(..)
                "(~, A, B)" => Compound(..)
                "<A --> B>" => Compound(..)
                "<A <-> B>" => Compound(..)
                "<A ==> B>" => Compound(..)
                "<A <=> B>" => Compound(..)
                // 多元
                "{A}" => Compound(..)
                "[A]" => Compound(..)
                "(&, A)" => Compound(..)
                "(|, A)" => Compound(..)
                "(*, A)" => Compound(..)
                r"(&&, A)" => Compound(..)
                r"(||, A)" => Compound(..)
                // 多元索引
                r"(/, R, _)" => Compound(..)
                r"(\, R, _)" => Compound(..)
            }
            ok!()
        }

        #[test]
        fn is_placeholder() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期
                macro is_placeholder($( $s:literal => $expected:expr )*) {
                    asserts! {$(
                        term!($s).is_placeholder() => $expected,
                    )*}
                }
                // 占位符
                "_" => true
                // 原子词项
                "A" => false
                "$A" => false
                "#A" => false
                "?A" => false
                // 复合词项
                "{A}" => false
                "[A]" => false
                "(&, A)" => false
                "(|, A)" => false
                "(-, A, B)" => false
                "(~, A, B)" => false
                "(*, A)" => false
                r"(/, R, _)" => false
                r"(\, R, _)" => false
                r"(&&, A)" => false
                r"(||, A)" => false
                r"(--, A)" => false
                // 陈述
                "<A --> B>" => false
                "<A <-> B>" => false
                "<A ==> B>" => false
                "<A <=> B>" => false
            }
            ok!()
        }

        /// 🎯仅测试其返回值为二元组
        #[test]
        fn id_comp() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串
                macro id_comp($($s:literal)*) {
                    asserts! {$(
                        term!($s).id_comp() => @(&_, &_),
                    )*}
                }
                // 占位符
                "_"
                // 原子词项
                "A"
                "$A"
                "#A"
                "?A"
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

        /// 🎯仅测试其返回值为二元组
        #[test]
        fn id_comp_mut() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串
                macro id_comp_mut($($s:literal)*) {
                    asserts! {$(
                        term!($s).id_comp_mut() => @(&mut _, &mut _),
                    )*}
                }
                // 占位符
                "_"
                // 原子词项
                "A"
                "$A"
                "#A"
                "?A"
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
        fn contain_type() -> AResult {
            macro_once! {
                // * 🚩模式：含有的类型 in 词项字符串
                macro contain_type($($expected:ident in $s:literal)*) {
                    asserts! {$(
                        term!($s).contain_type($expected)
                    )*}
                }
                // 复合词项
                WORD in "{A}"
                WORD in "[A]"
                WORD in "(&, A)"
                WORD in "(|, A)"
                WORD in "(-, A, B)"
                WORD in "(~, A, B)"
                WORD in "(*, A)"
                WORD in r"(/, R, _)"
                WORD in r"(\, R, _)"
                WORD in r"(&&, A)"
                WORD in r"(||, A)"
                WORD in r"(--, A)"

                VAR_INDEPENDENT in "{$A}"
                VAR_INDEPENDENT in "[$A]"
                VAR_INDEPENDENT in "(&, $A)"
                VAR_INDEPENDENT in "(|, $A)"
                VAR_INDEPENDENT in "(-, $A, B)"
                VAR_INDEPENDENT in "(~, $A, B)"
                VAR_INDEPENDENT in "(*, $A)"
                VAR_INDEPENDENT in r"(/, $R, _)"
                VAR_INDEPENDENT in r"(\, $R, _)"
                VAR_INDEPENDENT in r"(&&, $A)"
                VAR_INDEPENDENT in r"(||, $A)"
                VAR_INDEPENDENT in r"(--, $A)"

                PRODUCT_OPERATOR in "{(*, A)}"
                PRODUCT_OPERATOR in "[(*, A)]"
                PRODUCT_OPERATOR in "(&, (*, A))"
                PRODUCT_OPERATOR in "(|, (*, A))"
                PRODUCT_OPERATOR in "(-, (*, A), B)"
                PRODUCT_OPERATOR in "(~, (*, A), B)"
                PRODUCT_OPERATOR in "(*, (*, A))"
                PRODUCT_OPERATOR in r"(/, (*, R), _)"
                PRODUCT_OPERATOR in r"(\, (*, R), _)"
                PRODUCT_OPERATOR in r"(&&, (*, A))"
                PRODUCT_OPERATOR in r"(||, (*, A))"
                PRODUCT_OPERATOR in r"(--, (*, A))"

                // 陈述
                WORD in "<A --> B>"
                WORD in "<A <-> B>"
                WORD in "<A ==> B>"
                WORD in "<A <=> B>"

                INHERITANCE_RELATION in "<<A --> B> --> <A --> B>>"
                SIMILARITY_RELATION in "<<A <-> B> <-> <A <-> B>>"
                IMPLICATION_RELATION in "<<A ==> B> ==> <A ==> B>>"
                EQUIVALENCE_RELATION in "<<A <=> B> <=> <A <=> B>>"
            }
            ok!()
        }

        /// 🎯类型相等，组分相配
        #[test]
        fn structural_match() -> AResult {
            macro_once! {
                // * 🚩模式：被匹配的 ⇒ 用于匹配的
                macro assert_structural_match($($term1:literal => $term2:literal)*) {
                    asserts! {$(
                        term!($term1).structural_match(&term!($term2))
                    )*}
                }
                // 常规 //
                // 占位符
                "_" => "__"
                // 原子词项
                "A" => "a"
                "$A" => "$a"
                "#A" => "#a"
                "?A" => "?a"
                // 复合词项
                "{A}" => "{a}"
                "[A]" => "[a]"
                "(&, A)" => "(&, a)"
                "(|, A)" => "(|, a)"
                "(-, A, B)" => "(-, a, b)"
                "(~, A, B)" => "(~, a, b)"
                "(*, A)" => "(*, a)"
                r"(/, R, _)" => r"(/, r, _)"
                r"(\, R, _)" => r"(\, r, _)"
                r"(&&, A)" => r"(&&, a)"
                r"(||, A)" => r"(||, a)"
                r"(--, A)" => r"(--, a)"
                // 陈述
                "<A --> B>" => "<a --> b>"
                "<A <-> B>" => "<a <-> b>"
                "<A ==> B>" => "<a ==> b>"
                "<A <=> B>" => "<a <=> b>"
                // 可交换（⚠️只判断一层） //
                "{A, B, C}" => "{0, 1, 2}"
                "{A, B, [C]}" => "{0, 1, [2]}"
                "{A, {B, C, D}, [E]}" => "{{0, 1, 2}, 1, [2]}"
            }
            ok!()
        }

        #[test]
        fn fmt() -> AResult {
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
                "$A" => "$1" // ! 🚩【2024-06-13 23:53:31】现在「变量词项」会被重新命名
                "#A" => "#1" // ! 🚩【2024-06-13 23:53:31】现在「变量词项」会被重新命名
                "?A" => "?1" // ! 🚩【2024-06-13 23:53:31】现在「变量词项」会被重新命名
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
            ok!()
        }

        #[test]
        fn for_each_atom() -> AResult {
            fn test(term: Term, expected: &[Term]) {
                // 构造列表
                let mut v = vec![];
                // 遍历，复制，添加
                term.for_each_atom(&mut |t| v.push(t.clone()));
                // 断言
                assert_eq!(v, expected);
            }
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期词项字符串序列
                macro for_each_atom($($term:literal => [ $($expected:expr),* ] )*) {
                    $( test(term!($term), &term!([ $($expected),* ])); )*
                }
                // 简单情况（一层） //
                // 占位符
                "_" => ["_"]
                // 原子词项
                "A" => ["A"]
                "$A" => ["$A"]
                "#A" => ["#A"]
                "?A" => ["?A"]
                // 复合词项
                "{A, B}" => ["A", "B"]
                "[A, B]" => ["A", "B"]
                "(&, A, B)" => ["A", "B"]
                "(|, A, B)" => ["A", "B"]
                "(-, A, B)" => ["A", "B"]
                "(~, A, B)" => ["A", "B"]
                "(*, A, B)" => ["A", "B"]
                r"(/, R, _)" => ["R", "_"] // ! ⚠️【2024-06-13 17:47:14】现在会包含占位符了
                r"(\, R, _)" => ["R", "_"]
                r"(/, R, _, A)" => ["R", "_", "A"]
                r"(\, R, _, A)" => ["R", "_", "A"]
                r"(&&, A, B)" => ["A", "B"]
                r"(||, A, B)" => ["A", "B"]
                r"(--, A)" => ["A"]
                // 陈述
                "<A --> B>" => ["A", "B"]
                "<A <-> B>" => ["A", "B"]
                "<A ==> B>" => ["A", "B"]
                "<A <=> B>" => ["A", "B"]
                // 复杂情况 //
                // 复合词项后置，同时递归深入
                "(&&, A, B, [C, D])" => ["A", "B", "C", "D"]
                "<(--, (--, (--, (--, (--, (--, (--, (--, A)))))))) ==> <(-, B, C) --> (*, (*, (*, (*, (*, D)))))>>" => ["A", "B", "C", "D"]
                "<<A --> B> ==> <C --> D>>" => ["A", "B", "C", "D"]
            }
            ok!()
        }

        // TODO: 【2024-06-16 12:40:20】增加「判等⇔排序」的测试
    }

    /// 测试 / [`TermComponents`]
    mod term_components {
        use super::*;
        use nar_dev_utils::macro_once;

        /// 测试/长度
        #[test]
        fn len() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期结果
                macro asserts_len($( $term:literal => $s:expr )*) {
                    asserts! { $( term!($term).components.len() => $s )* }
                }
                // 平常情况
                "B" => 0
                "?quine" => 0
                "<A --> B>" => 2
                "(*, {SELF}, x, y)" => 3
                "(--, [good])" => 1
                // 像：占位符现已计入
                "(/, A, _, B)" => 3
                // 集合：缩并
                "[2, 1, 0, 0, 1, 2]" => 3
            }
            ok!()
        }

        /// 测试/判空
        #[test]
        fn is_empty() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期结果
                macro is_empty($($term:literal => $expected:expr)*) {
                    asserts! { $( term!($term).components.is_empty() => $expected )* }
                }
                "B" => true
                "?quine" => true
                "<A --> B>" => false
                "(*, {SELF}, x, y)" => false
                "(--, [good])" => false
                "(/, A, _, B)" => false
                "[2, 1, 0, 0, 1, 2]" => false
            }
            ok!()
        }

        /// 测试/获取
        #[test]
        fn get() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串.索引 ⇒ 预期结果
                macro get($($s:literal . $i:expr => $expected:expr)*) {
                    asserts! { $(
                        term!($s).components.get($i) => $expected
                    )* }
                }
                // 平常情况
                "B".0 => None
                "?quine".0 => None
                "<A --> B>".0 => Some(&term!("A"))
                "<A --> B>".1 => Some(&term!("B"))
                "<A --> B>".2 => None
                "{SELF}".0 => Some(&term!("SELF"))
                "{SELF}".1 => None
                "(*, {SELF}, x, y)".0 => Some(&term!("{SELF}"))
                "(*, {SELF}, x, y)".1 => Some(&term!("x"))
                "(*, {SELF}, x, y)".2 => Some(&term!("y"))
                "(*, {SELF}, x, y)".3 => None
                "(--, [good])".0 => Some(&term!("[good]"))
                "(--, [good])".1 => None
                // 像：【2024-06-13 17:50:45】占位符现已计入
                "(/, A, _, B)".0 => Some(&term!("A"))
                "(/, A, _, B)".1 => Some(&term!("_")) // ! 【2024-06-13 17:51:45】构造占位符目前是被允许的
                "(/, A, _, B)".2 => Some(&term!("B"))
                "(/, A, _, B)".3 => None
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]".0 => Some(&term!("0"))
                "[2, 1, 0, 0, 1, 2]".1 => Some(&term!("1"))
                "[2, 1, 0, 0, 1, 2]".2 => Some(&term!("2"))
                "[2, 1, 0, 0, 1, 2]".3 => None
            }
            ok!()
        }

        /// 测试/获取
        #[test]
        fn get_unchecked() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串.索引 ⇒ 预期结果
                macro get_unchecked($($s:literal . $i:expr => $expected:expr)*) {
                    unsafe { asserts! { $(
                        term!($s).components.get_unchecked($i) => $expected
                    )* } }
                }
                // 平常情况
                "<A --> B>".0 => &term!("A")
                "<A --> B>".1 => &term!("B")
                "{SELF}".0 => &term!("SELF")
                "(*, {SELF}, x, y)".0 => &term!("{SELF}")
                "(*, {SELF}, x, y)".1 => &term!("x")
                "(*, {SELF}, x, y)".2 => &term!("y")
                "(--, [good])".0 => &term!("[good]")
                // 像：【2024-06-13 17:50:45】占位符现已计入
                "(/, A, _, B)".0 => &term!("A")
                "(/, A, _, B)".1 => &term!("_")
                "(/, A, _, B)".2 => &term!("B")
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]".0 => &term!("0")
                "[2, 1, 0, 0, 1, 2]".1 => &term!("1")
                "[2, 1, 0, 0, 1, 2]".2 => &term!("2")
            }
            ok!()
        }

        /// 测试/迭代器
        /// * 🚩转换为数组，然后跟数组比对
        #[test]
        fn iter() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期结果
                macro iter($($s:literal => $expected:expr)*) {
                    asserts! { $(
                        term!($s).components.iter().collect::<Vec<_>>() => $expected
                    )* }
                }
                // 平常情况
                "<A --> B>" => term!(["A", "B"]&)
                "{SELF}" => term!(["SELF"]&)
                "(*, {SELF}, x, y)" => term!(["{SELF}", "x", "y"]&)
                "(--, [good])" => term!(["[good]"]&)
                // 像：【2024-06-13 17:50:45】占位符现已计入
                "(/, A, _, B)" => term!(["A", "_", "B"]&)
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]" => term!(["0", "1", "2"]&)
            }
            ok!()
        }

        #[test]
        fn sort_dedup() -> AResult {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期结果
                macro sort_dedup($($s:literal => $expected:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term}");
                        // 重排词项
                        term.components = term.components.sort_dedup();
                        // 验证结果
                        let expected = term!($expected);
                        println!(" => {term}");
                        assert_eq!(term, expected);
                    )*
                }
                // 重排
                "(*, B, C, A)" => "(*, A, B, C)"
                "(*, 2, 1, 3)" => "(*, 1, 2, 3)"
                "(/, R, T, _, S)" => "(/, R, S, T, _)" // ! ⚠️【2024-06-13 17:53:13】占位符现已被计入
                "{[C], $B, A}" => "{A, $B, [C]}"
                "(*, あ, え, い, お, う)" => "(*, あ, い, う, え, お)"
                "(*, ア, エ, イ, オ, ウ)" => "(*, ア, イ, ウ, エ, オ)"
                "(*, 一, 丄, 七, 丁, 丂)" => "(*, 一, 丁, 丂, 七, 丄)"
                // 去重
                "(*, F, A, D, E, D)" => "(*, A, D, E, F)"
                "(*, 1, 1, 4, 5, 1, 4)" => "(*, 1, 4, 5)"
            }
            ok!()
        }

        // ! 以下函数已在 `Term` 中测试
        // * contain_type
        // * structural_match
    }
}
