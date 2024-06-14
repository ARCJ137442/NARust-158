//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯用于「制作词项」

use super::Term;

impl Term {
    /// 制作「词语」
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
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
