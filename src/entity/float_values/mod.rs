//! 有关「浮点数集合」
//! * 📄短浮点
//! * 📄预算值
//! * 📄真值

// 短浮点 `ShortFloat` | 内有导出宏定义
mod short_float;
pub use short_float::*;

// 预算值 `BudgetValue`
mod budget_value;
pub use budget_value::*;

// 真值 `TruthValue`
mod truth_value;
pub use truth_value::*;
