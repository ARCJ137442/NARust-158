//! 🎯复刻OpenNARS `nars.entity.Concept`
//! TODO: 着手开始复刻

use super::Item;

/// 模拟OpenNARS `nars.entity.Concept`
/// * 🚩【2024-05-04 17:28:30】「概念」首先能被作为「Item」使用
pub trait Concept: Item {}
