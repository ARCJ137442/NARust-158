//! 📄OpenNARS `nars.language.Statement`
//! * 📌NAL底层的「陈述」逻辑，对应`Statement`及其所有子类
//! * ⚠️不包括与记忆区有关的`make`系列方法
//! * ⚠️不包括只和语法解析有关的`isRelation`、`makeName`、`makeStatementName`等方法
//! * ✅【2024-06-14 14:53:10】基本完成方法复刻
//!
//! # 方法列表
//! 🕒最后更新：【2024-06-14 14:53:18】
//!
//! * `Statement`
//!   * `invalidStatement` => `is_invalid_statement`
//!   * `invalidReflexive`
//!   * `invalidPair`
//!   * `invalid`
//!   * `getSubject`
//!   * `getPredicate`
//!
//! # 📄OpenNARS
//!
//! A statement is a compound term, consisting of a subject, a predicate, and a relation symbol in between.
//! It can be of either first-order or higher-order.

use super::compound_term::CompoundTermRef;
use crate::io::symbols::*;
use crate::language::*;
use nar_dev_utils::{if_return, matches_or};
use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

impl Term {
    /// 🆕用于判断是否为「陈述词项」
    /// * 📄OpenNARS `instanceof Statement` 逻辑
    #[inline(always)]
    pub fn instanceof_statement(&self) -> bool {
        Self::is_statement_identifier(&self.identifier)
    }

    /// 🆕抽象出来的「标识符（对应的词项类型）是否『可交换』」
    /// * 🎯同时用于「词项属性」与「词项转换」
    ///   * 📄参见[`super::_dialect`]中的`reform_term`函数
    pub fn is_statement_identifier(identifier: &str) -> bool {
        matches!(
            identifier,
            // 四大主要系词
            INHERITANCE_RELATION
                | SIMILARITY_RELATION
                | IMPLICATION_RELATION
                | EQUIVALENCE_RELATION
                // ↓下边都是派生系词（实际上不会出现，OpenNARS也一样）
                | INSTANCE_RELATION
                | PROPERTY_RELATION
                | INSTANCE_PROPERTY_RELATION
        )
    }

    /// 🆕用于判断是否为「继承」
    /// * 📄OpenNARS`instanceof Inheritance`逻辑
    /// * 📝OpenNARS中「继承」与「实例」「属性」「实例属性」没有继承关系
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_inheritance(&self) -> bool {
        self.identifier == INHERITANCE_RELATION
    }

    /// 🆕用于判断是否为「相似」
    /// * 📄OpenNARS`instanceof Similarity`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_similarity(&self) -> bool {
        self.identifier == SIMILARITY_RELATION
    }

    /// 🆕用于判断是否为「蕴含」
    /// * 📄OpenNARS`instanceof Implication`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_implication(&self) -> bool {
        self.identifier == IMPLICATION_RELATION
    }

    /// 🆕用于判断是否为「等价」
    /// * 📄OpenNARS`instanceof Equivalence`逻辑
    /// * 🎯[`crate::inference::RuleTables`]推理规则分派
    #[inline(always)]
    pub fn instanceof_equivalence(&self) -> bool {
        self.identifier == EQUIVALENCE_RELATION
    }

    /// 🆕判断一个词项是否为「陈述词项」
    /// * 🚩判断其「内部元素」的个数是否为2
    pub fn is_statement(&self) -> bool {
        matches!(&self.components, TermComponents::Compound(terms) if terms.len() == 2)
    }

    /// 🆕将一个复合词项转换为「陈述词项」（不可变引用）
    /// * 🚩转换为Option
    #[must_use]
    pub fn as_statement(&self) -> Option<StatementRef> {
        matches_or!(
            ?self.components,
            TermComponents::Compound(ref terms) if terms.len() == 2
            => StatementRef {
                statement: self,
                subject: &terms[0],
                predicate: &terms[1],
            }
        )
    }

    /// 🆕用于判断词项是否为「指定类型的复合词项」，并尝试返回「复合词项」的引用信息
    /// * 📌包括陈述
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回不可变引用
    #[must_use]
    pub fn as_statement_type(&self, statement_class: impl AsRef<str>) -> Option<StatementRef> {
        matches_or! {
            ?self.as_statement(),
            Some(statement)
                // * 🚩标识符相等
                if statement_class.as_ref() == self.identifier()
                // * 🚩内部（类型相等）的复合词项
                => statement
        }
    }

    /// 🆕将一个复合词项转换为「陈述词项」（可变引用）
    /// * 🚩转换为Option
    #[must_use]
    pub fn as_statement_mut(&mut self) -> Option<StatementRefMut> {
        matches_or!(
            ?self.components,
            TermComponents::Compound(ref mut terms) if terms.len() == 2
            => StatementRefMut {
                // * 🚩均转换为裸指针
                subject: &mut terms[0] as *mut Term,
                predicate: &mut terms[1] as *mut Term,
                statement: self,
            }
        )
    }

    /// 🆕用于判断词项是否为「陈述」并解包其中的主项和谓项
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_statement_components(self) -> Option<[Term; 2]> {
        matches_or! {
            ?self.unwrap_compound_components(),
            // * 🚩匹配到（语句所作为的）复合词项，同时长度合规
            Some(terms) if terms.len() == 2
            // * 🚩返回内容
            => {
                // ? 💭后续或许能提取出一个统一的逻辑
                let mut terms = terms.into_vec();
                let predicate = terms.pop().expect("已经假定了长度为2");
                let subject = terms.pop().expect("已经假定了长度为2");
                [subject, predicate]
            }
        }
    }

    /// 🆕用于判断词项是否为「指定类型的陈述」，并解包其中的主项和谓项
    /// * 🚩模式匹配后返回一个[`Option`]，只在其为「符合指定类型的词项」时为[`Some`]
    /// * 🚩返回内部所有元素的所有权
    #[must_use]
    pub fn unwrap_statement_type_components(
        self,
        statement_class: impl AsRef<str>,
    ) -> Option<[Term; 2]> {
        matches_or! {
            ?self.unwrap_compound_type_components(statement_class),
            // * 🚩匹配到（语句所作为的）复合词项，同时长度合规
            Some(terms) if terms.len() == 2
            // * 🚩返回内容
            => {
                // ? 💭后续或许能提取出一个统一的逻辑
                let mut terms = terms.into_vec();
                let predicate = terms.pop().expect("已经假定了长度为2");
                let subject = terms.pop().expect("已经假定了长度为2");
                [subject, predicate]
            }
        }
    }
}

/// 为「复合词项」添加「转换到陈述」的方法
/// * 📌依据：陈述 ⊂ 复合词项
impl CompoundTermRef<'_> {
    /// 🆕判断一个复合词项是否为「陈述词项」
    /// * 🚩判断其「内部元素」的个数是否为2
    /// * 📌与[`Term::is_statement`]一致
    pub fn is_statement(&self) -> bool {
        self.components.len() == 2
    }

    /// 🆕将一个复合词项转换为「陈述词项」（不可变引用）
    /// * 🚩转换为Option
    /// * 📌与[`Term::as_statement`]一致
    pub fn as_statement(&self) -> Option<StatementRef> {
        matches_or!(
            ?self.components,
            [ref subject, ref predicate]
            => StatementRef {
                statement: self.inner,
                subject,
                predicate,
            }
        )
    }

    // ! ❌【2024-06-14 14:47:26】没必要添加一个额外的`unchecked`方法：可以使用`unwrap`现场解包
}

/// 为「复合词项」添加「转换到陈述」的方法（可变引用）
/// * 📌依据：陈述 ⊂ 复合词项
impl CompoundTermRefMut<'_> {
    /// 🆕将一个复合词项转换为「陈述词项」（可变引用）
    /// * 🚩转换为Option
    /// * 📌与[`Term::as_statement`]一致
    pub fn as_statement(&mut self) -> Option<StatementRef> {
        matches_or!(
            // * 📝此处必须内联`self.components()`，以告诉借用检查器「并非使用整个结构」
            // ! SAFETY: 此处保证对整体（整个复合词项）拥有引用
            ? unsafe { &mut *self.components },
            [ref mut subject, ref mut predicate]
            => StatementRef {
                statement: self.inner,
                subject,
                predicate,
            }
        )
    }

    // ! ❌【2024-06-14 14:47:26】没必要添加一个额外的`unchecked`方法：可以使用`unwrap`现场解包
}

/// 🆕作为「陈述引用」的词项类型
/// * 🎯在程序类型层面表示一个「陈述」（不可变引用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementRef<'a> {
    /// 陈述词项本身
    pub statement: &'a Term,
    /// 陈述词项的主项
    pub subject: &'a Term,
    /// 陈述词项的谓项
    pub predicate: &'a Term,
}

impl StatementRef<'_> {
    /// 📄OpenNARS `getSubject`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn subject(&self) -> &Term {
        self.subject
    }

    /// 📄OpenNARS `getPredicate`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn predicate(&self) -> &Term {
        self.predicate
    }

    /// 📄OpenNARS `invalidStatement`
    /// * ⚠️必须是「陈述」才能调用
    /// * 🎯检查「无效陈述」
    /// * 🎯基于AIKR，避免定义无用、冗余的陈述
    ///   * 📄如「永远成立」的「重言式」tautology
    /// * 📌无效案例：
    ///   * `<A --> A>`
    ///   * `<A --> [A]>`
    ///   * `<[A] --> A>`
    ///   * `<<A --> B> ==> <B --> A>>`
    ///
    /// # 📄OpenNARS
    ///
    /// Check the validity of a potential Statement. [To be refined]
    pub fn invalid_statement(subject: &Term, predicate: &Term) -> bool {
        if_return! {
            // 重言式⇒无效
            subject == predicate => true
            //自反性检查（双向）
            Self::invalid_reflexive(subject, predicate) => true
            Self::invalid_reflexive(predicate, subject) => true
        }
        // 都是陈述⇒进一步检查
        matches_or! {
            (subject.as_statement(), predicate.as_statement()),
            // 获取各自的主词、谓词，并检查是否相等
            // ! 禁止如下格式： <<A --> B> ==> <B --> A>>
            // * 📄ERR: !!! INVALID INPUT: parseTerm: <<A --> B> ==> <B --> A>> --- invalid statement
            // ? 💭【2024-04-24 15:04:44】目前尚未明确含义，可能是防止「重复推导」
            /* 📄OpenNARS源码：
            if ((subject instanceof Statement) && (predicate instanceof Statement)) {
                Statement s1 = (Statement) subject;
                Statement s2 = (Statement) predicate;
                Term t11 = s1.getSubject();
                Term t12 = s1.getPredicate();
                Term t21 = s2.getSubject();
                Term t22 = s2.getPredicate();
                if (t11.equals(t22) && t12.equals(t21)) {
                    return true;
                }
            } */
            (
                Some(StatementRef { subject:ss, predicate:sp,.. }),
                Some(StatementRef { subject:ps, predicate:pp,.. })
            ) if ss == pp && sp == ps => return  true,
            () // 无效案例⇒继续检查
        }
        // 检查完毕⇒否
        false
    }

    /// 📄OpenNARS `invalidReflexive`
    /// * 🚩主词项是「非像复合词项」并且包括另一词项
    ///
    /// # 📄OpenNARS
    ///
    /// Check if one term is identical to or included in another one, except in a reflexive relation
    pub fn invalid_reflexive(may_container: &Term, may_component: &Term) -> bool {
        /* 📄OpenNARS源码：
        if (!(t1 instanceof CompoundTerm)) {
            return false;
        }
        CompoundTerm com = (CompoundTerm) t1;
        if ((com instanceof ImageExt) || (com instanceof ImageInt)) {
            return false;
        }
        return com.containComponent(t2);
        */
        /* 📝原样转译的Rust代码：
        if_return! {
            !container.instanceof_compound() => false
            container.instanceof_image() => false
        }
        container.contain_component(maybe_component)
        */
        match may_container.as_compound() {
            // 仅在复合词项时继续检查
            Some(compound) => {
                !compound.inner.instanceof_image() && compound.contain_component(may_component)
            }
            None => false,
        }
    }

    /// 📄OpenNARS `invalidPair`
    /// * 📝总体逻辑：是否「一边包含独立变量，而另一边不包含」
    ///   * 💭可能是要「避免自由变量」
    /// * 🚩两边「包含独立变量」的情况不一致
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn invalid_pair(subject: &Term, predicate: &Term) -> bool {
        /* 📄OpenNARS源码：
        if (Variable.containVarI(s1) && !Variable.containVarI(s2)) {
            return true;
        } else if (!Variable.containVarI(s1) && Variable.containVarI(s2)) {
            return true;
        }
        return false; */
        subject.contain_var_i() != predicate.contain_var_i()
    }

    /// 📄OpenNARS `invalid`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn invalid(&self) -> bool {
        Self::invalid_statement(self.subject(), self.predicate())
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for StatementRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.statement.fmt(f)
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述引用」可以被看作是一个普通的词项
impl Deref for StatementRef<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.statement
    }
}

/// 🆕作为「陈述引用」的词项类型
/// * 🎯在程序类型层面表示一个「陈述」（可变引用）
/// * 📝【2024-06-15 17:08:26】目前「陈述可变引用」用处不大
///   * 📄OpenNARS中没有与之相关的独有方法（`Statement`类中没有可变的方法）
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StatementRefMut<'a> {
    /// 陈述词项本身
    statement: &'a mut Term,
    /// 陈述词项的主项
    subject: *mut Term,
    /// 陈述词项的谓项
    predicate: *mut Term,
}

impl StatementRefMut<'_> {
    /// 获取陈述整体
    pub fn statement(&mut self) -> &mut Term {
        self.statement
    }

    /// 📄OpenNARS `getSubject`
    /// * ⚠️此处对裸指针解引用
    ///   * 📄安全性保证同[`CompoundTermRefMut::components`]
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn subject(&mut self) -> &mut Term {
        // SAFETY: 同[`Compound::components`]
        unsafe { &mut *self.subject }
    }

    /// 📄OpenNARS `getPredicate`
    /// * ⚠️此处对裸指针解引用
    ///   * 📄安全性保证同[`CompoundTermRefMut::components`]
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    pub fn predicate(&mut self) -> &mut Term {
        // SAFETY: 同[`Compound::components`]
        unsafe { &mut *self.predicate }
    }

    /// 生成一个不可变引用
    /// * 🚩将自身的所有字段转换为不可变引用，然后构造一个「不可变引用」结构
    /// * 📌可变引用一定能转换成不可变引用
    /// * ⚠️与[`AsRef`]与[`Deref`]不同：此处需要返回所有权，而非对目标类型（[`Term`]）的引用
    ///   * ❌返回`&CompoundTermRef`会导致「返回临时变量引用」故无法使用
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    pub fn into_ref<'s>(self) -> StatementRef<'s>
    where
        Self: 's,
    {
        // * 🚩解引用前（在debug模式下）检查
        debug_assert!(self.statement.is_statement());
        // * 🚩传递引用 & 裸指针解引用
        StatementRef {
            statement: self.statement,
            // SAFETY: 自身相当于对词项的可变引用，同时所有字段均保证有效——那就一定能同时转换
            subject: unsafe { &*self.subject },
            // SAFETY: 自身相当于对词项的可变引用，同时所有字段均保证有效——那就一定能同时转换
            predicate: unsafe { &*self.predicate },
        }
    }

    /// 转换为「复合词项可变引用」
    /// * 🎯不通过额外的「类型判断」（从[`DerefMut`]中来）转换为「复合词项可变引用」
    /// * ❌【2024-06-15 16:37:07】危险：不能在此【只传引用】，否则将能在「拿出引用」的同时「使用自身」
    pub fn into_compound_ref<'s>(self) -> CompoundTermRefMut<'s>
    where
        Self: 's,
    {
        debug_assert!(self.is_statement());
        // SAFETY: 保证「陈述词项」一定从「复合词项」中来
        unsafe { self.statement.as_compound_mut_unchecked() }
    }
}

/// 转发「呈现」方法到「内部词项」
impl Display for StatementRefMut<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.statement.fmt(f)
    }
}

/// 可变引用 ⇒ 不可变引用
impl<'s> From<StatementRefMut<'s>> for StatementRef<'s> {
    #[inline]
    fn from(r: StatementRefMut<'s>) -> Self {
        r.into_ref()
    }
}

/// 陈述可变引用 ⇒ 复合词项可变引用
impl<'s> From<StatementRefMut<'s>> for CompoundTermRefMut<'s> {
    #[inline]
    fn from(r: StatementRefMut<'s>) -> Self {
        r.into_compound_ref()
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述可变引用」可以被看作是一个普通的词项
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
/// * ❌【2024-06-15 15:38:58】不能实现「自动解引用到不可变引用」
impl Deref for StatementRefMut<'_> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        self.statement
    }
}

/// 向词项本身的自动解引用
/// * 🎯让「陈述可变引用」可以被看作是一个普通的词项（可变引用）
/// * 📌【2024-06-15 15:08:55】安全性保证：在该引用结构使用「元素列表」时，独占引用不允许其再度解引用
impl DerefMut for StatementRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.statement
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{ok, util::AResult};
    use nar_dev_utils::{asserts, macro_once};

    macro_rules! statement {
        // 可变引用/新常量
        (mut $term:literal) => {
            statement!(mut term!($term))
        };
        // 可变引用/原有变量
        (mut $term:expr) => {
            $term.as_statement_mut().unwrap()
        };
        // 不可变引用
        ($term:literal) => {
            statement!(term!($term))
        };
        // 不可变引用
        ($term:expr) => {
            $term.as_statement().unwrap()
        };
    }

    mod statement_ref {
        use super::*;

        /// 陈述有效性
        /// * 🎯一并测试
        ///   * `invalid`
        ///   * `invalid_statement`
        ///   * `invalid_reflexive`
        ///   * `invalid_pair`
        #[test]
        fn invalid() -> AResult {
            asserts! {
                // 非法
                statement!("<A --> A>").invalid()
                statement!("<A --> [A]>").invalid()
                statement!("<[A] --> A>").invalid()
                statement!("<<A --> B> ==> <B --> A>>").invalid()
                // 合法
                !statement!("<A --> B>").invalid()
                !statement!("<A --> [B]>").invalid()
                !statement!("<[A] --> B>").invalid()
                !statement!("<<A --> B> ==> <B --> C>>").invalid()
                !statement!("<<A --> B> ==> <C --> A>>").invalid()
                !statement!("<<A --> B> ==> <C --> D>>").invalid()
            }
            ok!()
        }

        #[test]
        fn subject_predicate() -> AResult {
            macro_once! {
                // * 🚩模式：陈述 ⇒ [主词, 谓词]
                macro test($($statement:expr => [$subject:literal, $predicate:literal])*) {
                    asserts! {$(
                        statement!($statement).subject() => &term!($subject)
                        statement!($statement).predicate() => &term!($predicate)
                    )*}
                }
                "<A --> B>"             => ["A", "B"]
                "<あ ==> α>"            => ["あ", "α"]
                "<{SELF} --> [good]>"   => ["{SELF}", "[good]"]
                "<<a --> b> ==> {C}>"   => ["<a --> b>", "{C}"]
                "<$1 --> [$2]>"         => ["$1", "[$2]"]
                "<(*, 1, 2, 3) ==> 4>"  => ["(*, 1, 2, 3)", "4"]
                // ! 实例、属性、实例属性 ⇒ 继承
                "<A {-- B>"             => ["{A}",  "B"]
                "<A --] B>"             => [ "A",  "[B]"]
                "<A {-] B>"             => ["{A}", "[B]"]
            }
            ok!()
        }
    }

    mod statement_ref_mut {
        use super::*;

        #[test]
        fn subject_predicate() -> AResult {
            macro_once! {
                // * 🚩模式：陈述 ⇒ [主词, 谓词]
                macro test($($statement:expr => [$subject:literal, $predicate:literal])*) {
                    asserts! {$(
                        statement!(mut $statement).subject() => &term!($subject)
                        statement!(mut $statement).predicate() => &term!($predicate)
                    )*}
                }
                "<A --> B>"             => ["A", "B"]
                "<あ ==> α>"            => ["あ", "α"]
                "<{SELF} --> [good]>"   => ["{SELF}", "[good]"]
                "<<a --> b> ==> {C}>"   => ["<a --> b>", "{C}"]
                "<$1 --> [$2]>"         => ["$1", "[$2]"]
                "<(*, 1, 2, 3) ==> 4>"  => ["(*, 1, 2, 3)", "4"]
                // ! 实例、属性、实例属性 ⇒ 继承
                "<A {-- B>"             => ["{A}",  "B"]
                "<A --] B>"             => [ "A",  "[B]"]
                "<A {-] B>"             => ["{A}", "[B]"]
            }
            ok!()
        }

        #[test]
        fn to_ref() -> AResult {
            fn test(mut term: Term) {
                // * 🚩非陈述⇒返回 | 🎯检验「检验函数」
                if !term.is_statement() {
                    return;
                }
                // * 🚩构建陈述的可变引用
                let mut statement = term.as_statement_mut().expect("是陈述了还转换失败");
                // * 🚩测试/Deref
                assert!(!statement.as_statement().unwrap().invalid());
                // * 🚩假定陈述有效
                statement.subject().identifier = "".into();
                // * 🚩转换为不可变引用
                let statement = statement.into_ref();
                assert!(!statement.invalid());
            }
            macro_once! {
                macro test($($term:expr)*) {
                    $(test(term!($term));)*
                }
                // !
                "A"
                "A"
            }
            ok!()
        }
    }
}
