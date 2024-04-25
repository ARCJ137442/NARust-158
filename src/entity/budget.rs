//! 内置的预算值

/// 📄OpenNARS `nars.entity.BudgetValue`
///
/// A triple of priority (current), durability (decay), and quality (long-term average).
pub type Budget = (f64, f64, f64);
