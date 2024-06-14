//! 源自改版OpenNARS 1.5.8各class的特性
//! * ✨词项 `Term`
//! * ✨复合词项 `CompoundTerm`
//! * ✨变量 `Variable`
//! * ✨像 `Image`
//! * ✨陈述 `Statement`

// 词项
// * 📄OpenNARS `nars.language.Term`
mod term;

// 复合词项
// * 📄OpenNARS `nars.language.CompoundTerm`
mod compound_term;
pub use compound_term::*;

// 变量
// * 📄OpenNARS `nars.language.Variable`
pub mod variable;
pub use variable::*;

// 像
// * 📄OpenNARS `nars.language.ImageXXt`
mod image;

// 陈述
// * 📄OpenNARS `nars.language.Statement`
mod statement;
pub use statement::*;
