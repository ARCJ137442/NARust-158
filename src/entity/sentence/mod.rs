//! 复刻改版OpenNARS「语句」
//! * ❌不附带「真值」假定——仅「判断」具有「真值」属性

// 语句 `Sentence`
mod sentence_trait;
pub use sentence_trait::*;

// 判断句
mod judgement;
pub use judgement::*;

// 疑问句
mod question;
pub use question::*;

// 初代实现

mod sentence_v1;
pub use sentence_v1::*;

mod judgement_v1;
pub use judgement_v1::*;

mod question_v1;
pub use question_v1::*;
