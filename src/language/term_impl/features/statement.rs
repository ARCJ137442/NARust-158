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
    pub fn as_statement(&self) -> Option<StatementRef<'_>> {
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
    pub fn as_statement(&self) -> Option<StatementRef<'_>> {
        matches_or!(
            ?self.components,
            [ref subject, ref predicate]
            => StatementRef {
                statement: self.term,
                subject,
                predicate,
            }
        )
    }

    // ! ❌【2024-06-14 14:47:26】没必要添加一个额外的`unchecked`方法：可以使用`unwrap`现场解包

    // ! 🚩【2024-06-14 14:45:48】暂不添加「陈述可变引用」
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
    /// * 🚩通过「组分」得到
    /// * 📌【2024-04-24 14:56:33】因为实现方式的区别，无法确保「能够得到 主词/谓词」
    ///   * ⚠️必须在调用时明确是「陈述」，否则`panic`
    ///
    /// # 📄OpenNARS
    ///
    pub fn get_subject(&self) -> &Term {
        self.subject
    }

    /// 📄OpenNARS `getPredicate`
    ///
    /// # 📄OpenNARS
    ///
    pub fn get_predicate(&self) -> &Term {
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
                !compound.term.instanceof_image() && compound.contain_component(may_component)
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
    pub fn invalid(&self) -> bool {
        Self::invalid_statement(self.get_subject(), self.get_predicate())
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::asserts;

    macro_rules! statement {
        ($($t:tt)*) => {
            term!($($t)*).as_statement().unwrap()
        };
    }

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
    fn get_subject() -> AResult {
        asserts! {
            statement!("<A --> B>").get_subject() => &term!("A")
            statement!("<あ --> B>").get_subject() => &term!("あ")
            statement!("<{SELF} --> B>").get_subject() => &term!("{SELF}")
            statement!("<<a --> b> --> B>").get_subject() => &term!("<a --> b>")
            statement!("<$1 --> B>").get_subject() => &term!("$1")
            statement!("<(*, 1, 2, 3) --> B>").get_subject() => &term!("(*, 1, 2, 3)")
        }
        ok!()
    }

    #[test]
    fn get_predicate() -> AResult {
        asserts! {
            statement!("<S --> A>").get_predicate() => &term!("A")
            statement!("<S --> あ>").get_predicate() => &term!("あ")
            statement!("<S --> {SELF}>").get_predicate() => &term!("{SELF}")
            statement!("<S --> <a --> b>>").get_predicate() => &term!("<a --> b>")
            statement!("<S --> $1>").get_predicate() => &term!("$1")
            statement!("<S --> (*, 1, 2, 3)>").get_predicate() => &term!("(*, 1, 2, 3)")
        }
        ok!()
    }
}
