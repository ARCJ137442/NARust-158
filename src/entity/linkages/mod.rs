//! 包括「词项链」与「任务链」等结构
//! * 📝用于表示「内容相关关系」
//! * 📝亦为NARS控制机制的一部分

// 基石：T链接
mod t_link;
pub use t_link::*;

// T链接的基础模板（其特化变种亦用作「词项链模板」）
mod t_linkage;
pub use t_linkage::*;

// 词项链
mod term_link;
pub use term_link::*;

// 任务链
mod task_link;
pub use task_link::*;
