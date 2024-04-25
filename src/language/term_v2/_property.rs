//! 实现 / 属性（内建）
//! * 🎯非OpenNARS所定义之「属性」「方法」
//!   * 📌至少并非OpenNARS原先所定义的

use nar_dev_utils::macro_once;

use super::*;

/// 手动实现「判等」逻辑
/// * 📄OpenNARS `Term.equals` 方法
/// * 🎯不让判等受各类「临时变量/词法无关的状态变量」的影响
///   * 📄`is_constant`字段
impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        macro_once! {
            // 宏：逐个字段比较相等
            // * 🎯方便表示、修改「要判等的字段」
            macro eq_fields($this:ident => $other:ident; $($field:ident)*) {
                $( $this.$field == $other.$field )&&*
            }
            // 判等逻辑
            self => other;
            identifier
            components
        }
    }
}

/// 手动实现「散列」逻辑
/// * 🎯在手动实现「判等」后，无法自动实现[`Hash`]（只能考虑到字段）
/// * 📄OpenNARS `hashCode`：直接使用其（词法上）唯一的「名称」作为依据
///   * ⚠️此处采取更本地化的做法：只散列化与之相关的字段，而无需调用字符串格式化函数
impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
        self.components.hash(state);
    }
}

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

    /// 判断其是否为「占位符」
    /// * 🎯【2024-04-21 01:04:17】在「词法折叠」中首次使用
    pub fn is_placeholder(&self) -> bool {
        self.identifier == PLACEHOLDER
    }

    /// 快捷获取「标识符-组分」二元组
    /// * 🎯用于很多地方的「类型匹配」
    pub fn id_comp(&self) -> (&str, &TermComponents) {
        (&self.identifier, &*self.components)
    }

    /// 快捷获取「标识符-组分」二元组，并提供可变机会
    /// * 🚩【2024-04-21 00:59:20】现在正常返回其两重可变引用
    /// * 📝【2024-04-21 00:58:58】当「标识符」为「静态字串」时，不能对其内部的`&str`属性进行修改
    ///   * 📌使用`&mut &str`会遇到生命周期问题
    ///   * 📌实际上「修改类型」本身亦不常用
    pub fn id_comp_mut(&mut self) -> (&mut str, &mut TermComponents) {
        (&mut self.identifier, &mut *self.components)
    }

    /// 判断「是否包含指定类型的词项」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    pub fn contain_type(&self, identifier: &str) -> bool {
        self.identifier == identifier || self.components.contain_type(identifier)
    }

    /// 判断和另一词项是否「结构匹配」
    /// * 🎯变量替换中的模式匹配
    /// * 🚩类型匹配 & 组分匹配
    /// * ⚠️非递归：不会递归比较「组分是否对应匹配」
    #[inline(always)]
    pub fn structural_match(&self, other: &Self) -> bool {
        self.get_class() == other.get_class() && self.components.structural_match(&other.components)
    }
}

/// 实现[`Display`]
/// * 🎯调试时便于展现内部结构
/// * ⚡性能友好
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_name())
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
            Empty | Named(..) => 0,
            // 固定数目
            Unary(..) => 1,
            Binary(..) => 2,
            // 不定数目
            Multi(terms) | MultiIndexed(_, terms) => terms.len(),
        }
    }

    /// 获取「组分是否为空」
    /// * 🎯自clippy提示而设
    pub fn is_empty(&self) -> bool {
        use TermComponents::*;
        match self {
            // 一定空
            Empty | Named(..) => true,
            // 一定非空
            Unary(..) | Binary(..) => false,
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => terms.is_empty(),
        }
    }

    /// 获取指定位置的组分（不一定有）
    /// * ⚠️对于「带索引序列」不受「索引」影响
    ///   * 📄对「像」不受「像占位符」影响
    pub fn get(&self, index: usize) -> Option<&Term> {
        use TermComponents::*;
        match (self, index) {
            // 无组分
            (Empty | Named(..), _) => None,
            // 固定数目 @ 固定索引
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => Some(term),
            // 不定数目
            (Multi(terms) | MultiIndexed(_, terms), _) => terms.get(index),
            // 其它情况⇒无
            _ => None,
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
        match (self, index) {
            // 固定数目
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => term,
            // 不定数目
            (Multi(terms) | MultiIndexed(_, terms), _) => terms.get_unchecked(index),
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
            Empty | Named(..) => Box::new(None.into_iter()),
            // 一定非空
            Unary(term) => Box::new([term].into_iter()),
            Binary(term1, term2) => Box::new([term1, term2].into_iter()),
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter()),
        };
        b
    }

    /// 获取其中「所有元素」的迭代器（可变引用）
    /// * 🚩返回一个迭代器，迭代其中所有「元素」
    /// * 🎯词项的「变量代入」替换
    /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
    /// * ⚠️【2024-04-25 14:50:41】不打算开放：后续若经此改变内部元素，将致「顺序混乱」等问题
    ///   * case: 对「可交换词项」，在更改后若未重排顺序，可能会破坏可交换性
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Term> {
        use TermComponents::*;
        // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
        let b: Box<dyn Iterator<Item = &mut Term>> = match self {
            // 一定空
            Empty | Named(..) => Box::new(None.into_iter()),
            // 一定非空
            Unary(term) => Box::new([term].into_iter()),
            Binary(term1, term2) => Box::new([term1, term2].into_iter()),
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter_mut()),
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
    /// * ⚠️「像占位符」不参与排序：不会影响到「像占位符」的位置
    pub(crate) fn sort_dedup(&mut self) {
        use TermComponents::*;
        match self {
            // 零元 | 一元 ⇒ 不排序
            Empty | Named(..) | Unary(..) => (),
            // 二元 ⇒ 排序内部词项，但不去重
            Binary(term1, term2) => {
                if term1 > term2 {
                    std::mem::swap(term1, term2);
                }
                // ❌【2024-04-25 15:00:34】使用临时数组进行重排，会导致引用失效
                // // 使用临时数组进行重排
                // let [new_term1, new_term2] = manipulate!(
                //     [term1, term2]
                //   => .sort()
                // );
                // // 重排后重新赋值
                // *term1 = *new_term1;
                // *term2 = *new_term2;
            }
            // 不定数目⇒直接对数组重排并去重
            Multi(terms) | MultiIndexed(_, terms) => {
                // 重排
                terms.sort();
                // 去重
                terms.dedup()
            }
        }
    }

    /// 尝试向其中添加元素
    /// * ⚠️【2024-04-25 14:48:37】默认作为**有序**容器处理
    ///   * 📌其「可交换性」交由「词项」处理
    ///   * 📌对所谓「可交换词项」不会在此重新排序
    /// * 🚩始终作为其内的「组分」添加，没有「同类⇒组分合并」的逻辑
    /// * 🚩返回「是否添加成功」
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub(super) fn add(&mut self, term: Term) -> bool {
        use TermComponents::*;
        match self {
            // 固定数目的词项⇒必然添加失败
            Empty | Named(..) | Unary(..) | Binary(..) => false,
            // 不定数目⇒添加
            Multi(terms) | MultiIndexed(_, terms) => {
                terms.push(term);
                true
            }
        }
    }

    /// 尝试向其中删除元素
    /// * 🚩始终作为其内的「组分」删除，没有「同类⇒删除其中所有组分」的逻辑
    /// * 🚩返回「是否删除成功」
    /// * ⚠️只会移除一个
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub(super) fn remove(&mut self, term: &Term) -> bool {
        use TermComponents::*;
        match self {
            // 固定数目的词项⇒必然删除失败
            Empty | Named(..) | Unary(..) | Binary(..) => false,
            // 不定数目⇒尝试删除
            Multi(terms) | MultiIndexed(_, terms) => match terms.iter().position(|t| t == term) {
                // 找到⇒删除
                Some(index) => {
                    terms.remove(index);
                    true
                }
                // 未找到⇒返回false
                None => false,
            },
        }
    }

    /// 尝试向其中替换元素
    /// * 🚩始终作为其内的「组分」替换
    /// * 🚩返回「是否替换成功」
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub(super) fn replace(&mut self, index: usize, new: Term) -> bool {
        use TermComponents::*;
        match (self, index) {
            // 无组分
            (Empty | Named(..), _) => false,
            // 固定数目 @ 固定索引
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => {
                *term = new;
                true
            }
            // 不定数目 & 长度保证
            (Multi(terms) | MultiIndexed(_, terms), _) if index < terms.len() => {
                terms[index] = new;
                true
            }
            // 其它情况⇒无
            _ => false,
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
            // 同类型 / 空 | 同类型 / 一元 | 同类型 / 二元
            (Empty | Named(..), Empty | Named(..))
            | (Unary(..), Unary(..))
            | (Binary(..), Binary(..)) => true,
            // 同类型 / 多元
            (Multi(terms1), Multi(terms2)) => terms1.len() == terms2.len(),
            (MultiIndexed(i1, terms1), MultiIndexed(i2, terms2)) => {
                i1 == i2 && terms1.len() == terms2.len()
            }
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
    use anyhow::Result;
    use nar_dev_utils::asserts;

    /// 测试 / [`Term`]
    mod term {
        use super::*;
        use nar_dev_utils::macro_once;

        #[test]
        fn eq() -> Result<()> {
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
            Ok(())
        }

        /// 测试 / 散列
        /// * 🚩【2024-04-25 09:24:58】仅测试其「可散列化」
        #[test]
        fn hash() -> Result<()> {
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
            Ok(())
        }

        #[test]
        fn identifier() -> Result<()> {
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
            Ok(())
        }

        #[test]
        fn components() -> Result<()> {
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
                "A" => Named(..)
                "$A" => Named(..)
                "#A" => Named(..)
                "?A" => Named(..)
                // 一元
                "(--, A)" => Unary(..)
                // 二元
                "(-, A, B)" => Binary(..)
                "(~, A, B)" => Binary(..)
                "<A --> B>" => Binary(..)
                "<A <-> B>" => Binary(..)
                "<A ==> B>" => Binary(..)
                "<A <=> B>" => Binary(..)
                // 多元
                "{A}" => Multi(..)
                "[A]" => Multi(..)
                "(&, A)" => Multi(..)
                "(|, A)" => Multi(..)
                "(*, A)" => Multi(..)
                r"(&&, A)" => Multi(..)
                r"(||, A)" => Multi(..)
                // 多元索引
                r"(/, R, _)" => MultiIndexed(..)
                r"(\, R, _)" => MultiIndexed(..)
            }
            Ok(())
        }

        #[test]
        fn is_placeholder() -> Result<()> {
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
            Ok(())
        }

        /// 🎯仅测试其返回值为二元组
        #[test]
        fn id_comp() -> Result<()> {
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
            Ok(())
        }

        /// 🎯仅测试其返回值为二元组
        #[test]
        fn id_comp_mut() -> Result<()> {
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
            Ok(())
        }

        #[test]
        fn contain_type() -> Result<()> {
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
            Ok(())
        }

        /// 🎯类型相等，组分相配
        #[test]
        fn structural_match() -> Result<()> {
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
            Ok(())
        }

        #[test]
        fn fmt() -> Result<()> {
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
    }

    /// 测试 / [`TermComponents`]
    mod term_components {
        use super::*;
        use nar_dev_utils::macro_once;

        /// 测试/长度
        #[test]
        fn len() -> Result<()> {
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
                // 像：占位符不算
                "(/, A, _, B)" => 2
                // 集合：缩并
                "[2, 1, 0, 0, 1, 2]" => 3
            }
            Ok(())
        }

        /// 测试/判空
        #[test]
        fn is_empty() -> Result<()> {
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
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get() -> Result<()> {
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
                // 像：占位符不算
                "(/, A, _, B)".0 => Some(&term!("A"))
                "(/, A, _, B)".1 => Some(&term!("B"))
                "(/, A, _, B)".2 => None
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]".0 => Some(&term!("0"))
                "[2, 1, 0, 0, 1, 2]".1 => Some(&term!("1"))
                "[2, 1, 0, 0, 1, 2]".2 => Some(&term!("2"))
                "[2, 1, 0, 0, 1, 2]".3 => None
            }
            Ok(())
        }

        /// 测试/获取
        #[test]
        fn get_unchecked() -> Result<()> {
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
                // 像：占位符不算
                "(/, A, _, B)".0 => &term!("A")
                "(/, A, _, B)".1 => &term!("B")
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]".0 => &term!("0")
                "[2, 1, 0, 0, 1, 2]".1 => &term!("1")
                "[2, 1, 0, 0, 1, 2]".2 => &term!("2")
            }
            Ok(())
        }

        /// 测试/迭代器
        /// * 🚩转换为数组，然后跟数组比对
        #[test]
        fn iter() -> Result<()> {
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
                // 像：占位符不算
                "(/, A, _, B)" => term!(["A", "B"]&)
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]" => term!(["0", "1", "2"]&)
            }
            Ok(())
        }

        /// 测试/可变迭代器
        /// * 🎯仅测试「可以修改」
        #[test]
        fn iter_mut() -> Result<()> {
            fn mutate(term: &mut Term) {
                // 改变词项标识符
                term.identifier = "MUTATED".to_string();
                // 检验是否改变
                assert!(term.identifier == "MUTATED");
            }
            macro_once! {
                // * 🚩模式：词项字符串
                macro iter_mut($($s:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term} => ");
                        // 遍历修改
                        term.components.iter_mut().for_each(mutate);
                        println!("{term}");
                    )*
                }
                // 平常情况
                "<A --> B>"
                "<<A --> B> --> <A --> B>>"
                "{SELF}"
                "(*, {SELF}, x, y)"
                "(--, [good])"
                // 像：占位符不算
                "(/, A, _, B)"
                // 集合：排序 & 缩并
                "[2, 1, 0, 0, 1, 2]"
            }
            Ok(())
        }

        #[test]
        fn sort_dedup() -> Result<()> {
            macro_once! {
                // * 🚩模式：词项字符串 ⇒ 预期结果
                macro sort_dedup($($s:literal => $expected:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term}");
                        // 重排词项
                        term.components.sort_dedup();
                        // 验证结果
                        let expected = term!($expected);
                        println!(" => {term}");
                        assert_eq!(term, expected);
                    )*
                }
                // 重排
                "(*, B, C, A)" => "(*, A, B, C)"
                "(*, 2, 1, 3)" => "(*, 1, 2, 3)"
                "(/, R, T, _, S)" => "(/, R, S, _, T)"
                "(*, あ, え, い, お, う)" => "(*, あ, い, う, え, お)"
                "(*, ア, エ, イ, オ, ウ)" => "(*, ア, イ, ウ, エ, オ)"
                "(*, 一, 丄, 七, 丁, 丂)" => "(*, 一, 丁, 丂, 七, 丄)"
                // 去重
                "(*, F, A, D, E, D)" => "(*, A, D, E, F)"
                "(*, 1, 1, 4, 5, 1, 4)" => "(*, 1, 4, 5)"
            }
            Ok(())
        }

        /// ! 不考虑「可交换性」这个「复合词项」`compound`才引入的概念
        /// * ⚠️因此只对「不可交换的词项」进行测试
        #[test]
        fn add() -> Result<()> {
            macro_once! {
                // * 🚩模式：词项字符串 (+ 附加词项字符串)... ⇒ 预期结果
                macro add($($s:literal $(+ $new:literal)* => $expected:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term}");
                        // 追加词项
                        $(
                            let new = term!($new);
                            print!(" + {new}");
                            term.add(new);
                        )*
                        // 验证结果
                        let expected = term!($expected);
                        println!(" => {term}");
                        assert_eq!(term, expected);
                    )*
                }
                // ! 此处不考虑「可交换词项」如「集合」「外延交」等
                // 平常情况
                "(*, SELF)" + "good" => "(*, SELF, good)"
                "(*, あ)" + "い" + "う" + "え" + "お" => "(*, あ, い, う, え, お)"
                "(*, x, y)" + "z" => "(*, x, y, z)"
                "(*, 你)" + "我" + "他" => "(*, 你, 我, 他)"
                "(*, 0, 1, 2)" + "3" => "(*, 0, 1, 2, 3)"
                // 像：占位符不算
                r"(/, A, _, B)" + "C" => r"(/, A, _, B, C)"
                r"(\, A, _, B)" + "C" => r"(\, A, _, B, C)"
                r"(\, 甲, _, 乙)" + "{丙}" + "<丁 ==> 戊>" => r"(\, 甲, _, 乙, {丙}, <丁 ==> 戊>)"
            }
            Ok(())
        }

        #[test]
        fn remove() -> Result<()> {
            macro_once! {
                // * 🚩模式：词项字符串 (- 附加词项字符串)... ⇒ 预期结果
                macro remove($($s:literal $(- $no:literal)* => $expected:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term}");
                        // 追加词项
                        $(
                            let no = term!($no);
                            print!(" - {no}");
                            term.remove(&no);
                        )*
                        // 验证结果
                        let expected = term!($expected);
                        println!(" => {term}");
                        assert_eq!(term, expected);
                    )*
                }
                // ! 此处不考虑「可交换词项」如「集合」「外延交」等
                // 平常情况
                "(*, SELF, good)" - "good" => "(*, SELF)"
                "(*, あ, い, う, え, お)" - "い" - "う" - "え" - "お" => "(*, あ)"
                "(*, x, y, z)" - "z" => "(*, x, y)"
                "(*, 你, 我, 他)" - "我" - "他" => "(*, 你)"
                "(*, 0, 1, 2, 3)" - "3" => "(*, 0, 1, 2)"
                // 像：占位符不算
                r"(/, A, _, B, C)" - "C" => r"(/, A, _, B)"
                r"(\, A, _, B, C)" - "C" => r"(\, A, _, B)"
                r"(\, 甲, _, 乙, {丙}, <丁 ==> 戊>)" - "{丙}" - "<丁 ==> 戊>" => r"(\, 甲, _, 乙)"
            }
            Ok(())
        }

        #[test]
        fn replace() -> Result<()> {
            macro_once! {
                // * 🚩模式：词项字符串[索引] = 新词项 ⇒ 预期结果
                macro replace($($s:literal [ $i:expr ] = $new:literal => $expected:literal)*) {
                    $(
                        // 构造词项
                        let mut term = term!($s);
                        print!("{term}");
                        // 替换词项
                        term.components.replace($i, term!($new));
                        // 验证结果
                        let expected = term!($expected);
                        println!(" => {term}");
                        assert_eq!(term, expected);
                    )*
                }
                // ! 此处不考虑「可交换词项」如「集合」「外延交」等
                // 平常情况
                "(*, SELF, bad)"[1] = "good" => "(*, SELF, good)"
                "(*, x, y, ζ)"[2] = "z" => "(*, x, y, z)"
                "(*, 逆, 我, 他)"[0] = "你" => "(*, 你, 我, 他)"
                // 像：占位符不算
                r"(/, a, _, B, C)"[0] = "A" => r"(/, A, _, B, C)"
                r"(\, A, _, β, C)"[1] = "B" => r"(\, A, _, B, C)"
                r"(\, 甲, _, 乙, {饼}, <丁 ==> 戊>)"[2] = "{丙}" => r"(\, 甲, _, 乙, {丙}, <丁 ==> 戊>)"
            }
            Ok(())
        }

        // ! 以下函数已在 `Term` 中测试
        // * contain_type
        // * structural_match
    }
}
