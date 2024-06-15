//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯复刻原OpenNARS 1.5.8的`make`系列方法
//! * 🚩构造词项前，
//!   * 检查其合法性
//!   * 简化其表达
//! * 🎯用于「制作词项」

use super::{CompoundTermRef, StatementRef, Term};
use crate::io::symbols::*;

impl Term {
    /* Word */

    /// 制作「词语」
    #[inline]
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
    }

    /* Variable */

    /// 制作「独立变量」
    #[inline]
    pub fn make_var_i(id: usize) -> Term {
        Term::new_var_i(id)
    }

    /// 制作「非独变量」
    #[inline]
    pub fn make_var_d(id: usize) -> Term {
        Term::new_var_d(id)
    }

    /// 制作「查询变量」
    #[inline]
    pub fn make_var_q(id: usize) -> Term {
        Term::new_var_q(id)
    }

    /// 制作「与现有变量类型相同」的变量
    /// * 🚩类型相同但编号不同
    /// * 🎯用于「变量推理」中的「重命名变量」
    #[inline]
    pub fn make_var_similar(from: &Term, id: impl Into<usize>) -> Term {
        Term::from_var_similar(from.identifier(), id)
    }

    /* CompoundTerm */

    /// 📄OpenNARS `public static Term makeCompoundTerm(CompoundTerm compound, ArrayList<Term> components)`
    pub fn make_compound_term(template: CompoundTermRef, components: Vec<Term>) -> Option<Term> {
        /* 📄OpenNARS
        if (compound instanceof ImageExt)
            // * 🚩外延像
            return makeImageExt(components, ((ImageExt) compound).getRelationIndex());
        else if (compound instanceof ImageInt)
            // * 🚩内涵像
            return makeImageInt(components, ((ImageInt) compound).getRelationIndex());
        else
            // * 🚩其它
            return makeCompoundTerm(compound.operator(), components); */
        let term = template.inner;
        if term.instanceof_image_ext() {
            Self::make_image_ext(components, template.get_placeholder_index())
        } else if term.instanceof_image_int() {
            Self::make_image_int(components, template.get_placeholder_index())
        } else {
            Self::make_compound_term_from_identifier(&term.identifier, components)
        }
    }

    pub fn make_compound_term_or_statement(
        template: CompoundTermRef,
        mut components: Vec<Term>,
    ) -> Option<Term> {
        match template.as_statement() {
            // * 🚩陈述模板
            Some(statement) => match &components.as_slice() {
                // * 🚩双元素
                &[_, _] => {
                    // * 🚩取出其中仅有的两个元素
                    let predicate = components.pop().unwrap();
                    let subject = components.pop().unwrap();
                    Self::make_statement(statement, subject, predicate)
                }
                // * 🚩其它⇒无
                _ => None,
            },
            // * 🚩复合词项⇒继续
            _ => Self::make_compound_term(template, components),
        }
    }

    /// 📄OpenNARS `public static Term makeCompoundTerm(String op, ArrayList<Term> arg)`
    pub fn make_compound_term_from_identifier(
        identifier: impl AsRef<str>,
        argument: Vec<Term>,
    ) -> Option<Term> {
        match identifier.as_ref() {
            SET_EXT_OPERATOR => Self::make_set_ext_arg(argument),
            SET_INT_OPERATOR => Self::make_set_int_arg(argument),
            DIFFERENCE_EXT_OPERATOR => Self::make_difference_ext_arg(argument),
            PRODUCT_OPERATOR => Self::make_product_arg(argument),
            IMAGE_EXT_OPERATOR => Self::make_image_ext_arg(argument),
            IMAGE_INT_OPERATOR => Self::make_image_int_arg(argument),
            NEGATION_OPERATOR => Self::make_negation_arg(argument),
            CONJUNCTION_OPERATOR => Self::make_conjunction_arg(argument),
            DISJUNCTION_OPERATOR => Self::make_disjunction_arg(argument),
            // * 🚩其它⇒未知/域外⇒空
            _ => None,
        }
    }

    pub fn make_set_ext_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_set_int_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_difference_ext_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_product_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_ext(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_ext_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_int(argument: Vec<Term>, placeholder_index: usize) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_int_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_conjunction_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_disjunction_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_negation_arg(argument: Vec<Term>) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    /* Statement */

    pub fn make_statement(template: StatementRef, subject: Term, predicate: Term) -> Option<Term> {
        todo!("// TODO: 有待复刻")
    }

    #[cfg(TODO)] // TODO: 有待复用
    /// 📄OpenNARS `Statement.makeSym`
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(TODO)] // TODO: 有待复用
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
}
