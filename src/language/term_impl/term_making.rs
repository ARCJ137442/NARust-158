//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯复刻原OpenNARS 1.5.8的`make`系列方法
//! * 🚩构造词项前，
//!   * 检查其合法性
//!   * 简化其表达
//! * 🎯用于「制作词项」

use super::{CompoundTermRef, Term};

impl Term {
    /// 制作「词语」
    #[inline]
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
    }

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

    /// 📄OpenNARS `public static Term makeCompoundTerm(CompoundTerm compound, ArrayList<Term> components)`
    pub fn make_compound_term(template: CompoundTermRef, components: Vec<Term>) -> Term {
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
        let term = template.term;
        if term.instanceof_image_ext() {
            Self::make_image_ext(components, template.get_placeholder_index())
        } else if term.instanceof_image_int() {
            Self::make_image_int(components, template.get_placeholder_index())
        } else {
            Self::make_compound_term_from_identifier(&term.identifier, components)
        }
    }

    /// 📄OpenNARS `public static Term makeCompoundTerm(String op, ArrayList<Term> arg)`
    pub fn make_compound_term_from_identifier(
        identifier: impl Into<String>,
        argument: Vec<Term>,
    ) -> Term {
        let identifier = identifier.into();
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_ext(argument: Vec<Term>, placeholder_index: usize) -> Term {
        todo!("// TODO: 有待复刻")
    }

    pub fn make_image_int(argument: Vec<Term>, placeholder_index: usize) -> Term {
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
