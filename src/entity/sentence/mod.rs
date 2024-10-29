//! 复刻改版OpenNARS「语句」
//! * ❌不附带「真值」假定——仅「判断」具有「真值」属性

// 标点 `Punctuation`
mod punctuation;
pub use punctuation::*;

// 语句 `Sentence`
mod sentence_trait;
pub use sentence_trait::*;

// 判断句
mod judgement;
pub use judgement::*;

// 目标句
mod goal;
pub use goal::*;

// 疑问句
mod question;
pub use question::*;

// 初代实现
mod impls;
pub use impls::*;
