//! 📄OpenNARS `nars.language.Statement`
//! * 📌NAL底层的「陈述」逻辑，对应`Statement`及其所有子类
//! * ⚠️不包括与记忆区有关的`make`系列方法
//! * ⚠️不包括只和语法解析有关的`isRelation`、`makeName`、`makeStatementName`等方法
//!
//! # 方法列表
//! 🕒最后更新：【2024-04-24 14:32:52】
//!
//! * `Statement`
//!   * `makeSym` => `new_sym_statement`
//!   * `invalidStatement` => `is_invalid_statement`
//!   * `invalidReflexive`
//!   * `invalidPair`
//!   * `invalid` => `invalid_statement`
//!   * `getSubject`
//!   * `getPredicate`
//!
//! # 📄OpenNARS
//!
//! A statement is a compound term, consisting of a subject, a predicate, and a relation symbol in between.
//! It can be of either first-order or higher-order.

use super::*;
use nar_dev_utils::if_return;

impl Term {
    /// 🆕用于判断是否为「陈述词项」
    /// * 📄OpenNARS `instanceof Statement` 逻辑
    pub fn instanceof_statement(&self) -> bool {
        matches!(
            self.identifier.as_str(),
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

    /// 📄OpenNARS `Statement.makeSym` 方法
    /// * 🚩通过使用「标识符映射」将「非对称版本」映射到「对称版本」
    /// * ⚠️目前只支持「继承」和「蕴含」，其它均会`panic`
    ///
    /// # 📄OpenNARS
    /// Make a symmetric Statement from given components and temporal information,
    /// called by the rules
    pub fn new_sym_statement(identifier: &str, subject: Term, predicate: Term) -> Self {
        match identifier {
            // 继承⇒相似
            INHERITANCE_RELATION => Term::new_similarity(subject, predicate),
            // 蕴含⇒等价
            IMPLICATION_RELATION => Term::new_equivalence(subject, predicate),
            // 其它⇒panic
            _ => unimplemented!("不支持的标识符：{identifier:?}"),
        }
    }

    /// 📄OpenNARS `invalidStatement` 方法
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
    pub fn is_invalid_statement(subject: &Term, predicate: &Term) -> bool {
        if_return! {
            // 重言式⇒无效
            subject == predicate => true
            //自反性检查（双向）
            Term::invalid_reflexive(subject, predicate) => true
            Term::invalid_reflexive(predicate, subject) => true
        }
        // 都是陈述⇒进一步检查
        if subject.instanceof_statement() && predicate.instanceof_statement() {
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
            if (subject.get_subject() == predicate.get_predicate())
                && (subject.get_predicate() == predicate.get_subject())
            {
                return true;
            }
        }
        // 检查完毕⇒否
        false
    }

    /// 📄OpenNARS `invalidReflexive` 方法
    /// * 🚩主词项是「非像复合词项」并且包括另一词项
    ///
    /// # 📄OpenNARS
    ///
    /// Check if one term is identical to or included in another one, except in a reflexive relation
    pub fn invalid_reflexive(container: &Term, maybe_component: &Term) -> bool {
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
        container.instanceof_compound()
            && !container.instanceof_image()
            && container.contain_component(maybe_component)
    }

    /// 📄OpenNARS `invalidPair` 方法
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

    /// 📄OpenNARS `invalid` 方法
    ///
    /// # 📄OpenNARS
    ///
    pub fn invalid_statement(&self) -> bool {
        Self::is_invalid_statement(self.get_subject(), self.get_predicate())
    }

    /// 📄OpenNARS `getSubject` 方法
    /// * 🚩通过「组分」得到
    /// * 📌【2024-04-24 14:56:33】因为实现方式的区别，无法确保「能够得到 主词/谓词」
    ///   * ⚠️必须在调用时明确是「陈述」，否则`panic`
    ///
    /// # 📄OpenNARS
    ///
    pub fn get_subject(&self) -> &Term {
        match &*self.components {
            TermComponents::Binary(subject, _) => subject,
            _ => panic!("尝试向「非陈述词项」获取主词"),
        }
    }

    /// 📄OpenNARS `getPredicate` 方法
    ///
    /// # 📄OpenNARS
    ///
    pub fn get_predicate(&self) -> &Term {
        match &*self.components {
            TermComponents::Binary(_, predicate) => predicate,
            _ => panic!("尝试向「非陈述词项」获取谓词"),
        }
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::{global::tests::AResult, ok};
    use nar_dev_utils::asserts;

    #[test]
    fn new_sym_statement() -> AResult {
        asserts! {
            // 继承⇒相似
            Term::new_sym_statement(INHERITANCE_RELATION, term!("A"), term!("B"))
                => term!("<A <-> B>")
            // 蕴含⇒等价
            Term::new_sym_statement(IMPLICATION_RELATION, term!("A"), term!("B"))
                => term!("<A <=> B>")
        }
        ok!()
    }

    /// 陈述有效性
    /// * 🎯一并测试
    ///   * `is_invalid_statement`
    ///   * `invalid_statement`
    ///   * `invalid_reflexive`
    ///   * `invalid_pair`
    #[test]
    fn invalid_statement() -> AResult {
        asserts! {
            // 非法
            term!("<A --> A>").invalid_statement()
            term!("<A --> [A]>").invalid_statement()
            term!("<[A] --> A>").invalid_statement()
            term!("<<A --> B> ==> <B --> A>>").invalid_statement()
            // 合法
            !term!("<A --> B>").invalid_statement()
            !term!("<A --> [B]>").invalid_statement()
            !term!("<[A] --> B>").invalid_statement()
            !term!("<<A --> B> ==> <B --> C>>").invalid_statement()
            !term!("<<A --> B> ==> <C --> A>>").invalid_statement()
            !term!("<<A --> B> ==> <C --> D>>").invalid_statement()
        }
        ok!()
    }

    #[test]
    fn get_subject() -> AResult {
        asserts! {
            term!("<A --> B>").get_subject() => &term!("A")
            term!("<あ --> B>").get_subject() => &term!("あ")
            term!("<{SELF} --> B>").get_subject() => &term!("{SELF}")
            term!("<<a --> b> --> B>").get_subject() => &term!("<a --> b>")
            term!("<$1 --> B>").get_subject() => &term!("$1")
            term!("<(*, 1, 2, 3) --> B>").get_subject() => &term!("(*, 1, 2, 3)")
        }
        ok!()
    }

    #[test]
    fn get_predicate() -> AResult {
        asserts! {
            term!("<S --> A>").get_predicate() => &term!("A")
            term!("<S --> あ>").get_predicate() => &term!("あ")
            term!("<S --> {SELF}>").get_predicate() => &term!("{SELF}")
            term!("<S --> <a --> b>>").get_predicate() => &term!("<a --> b>")
            term!("<S --> $1>").get_predicate() => &term!("$1")
            term!("<S --> (*, 1, 2, 3)>").get_predicate() => &term!("(*, 1, 2, 3)")
        }
        ok!()
    }
}
