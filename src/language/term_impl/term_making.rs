//! 📄OpenNARS `nars.language.MakeTerm`
//! * 🎯用于「制作词项」

use super::Term;

impl Term {
    /// 制作「词语」
    pub fn make_word(name: impl Into<String>) -> Term {
        Term::new_word(name)
    }
}
