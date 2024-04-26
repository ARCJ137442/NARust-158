//! 内置的预算值

use narsese::api::EvidentNumber;

/// 抽象的「预算」特征
/// * 🎯实现最大程度的抽象与通用
///   * 💭后续可以在底层用各种「证据值」替换，而不影响整个推理器逻辑
///  📄OpenNARS `nars.entity.BudgetValue`
///
/// A triple of priority (current), durability (decay), and quality (long-term average).
pub trait BudgetValue {
    /// 一种类型只可能有一种「证据值」
    type E: EvidentNumber;

    /// 获取优先级
    fn priority(&self) -> Self::E;

    /// 获取耐久度
    fn durability(&self) -> Self::E;

    /// 获取质量
    fn quality(&self) -> Self::E;

    /// 检查自身合法性
    /// * 📜分别检查`priority`、`durability`、`quality`的合法性
    fn check_valid(&self) -> bool {
        self.priority().is_valid() && self.durability().is_valid() && self.quality().is_valid()
    }
    // TODO: 复现更多所需功能
}

/// 预算[`Budget`]的可变版本
/// * 📌允许修改内部值
///   * ⚠️尽可能在修改内部值时，保证值合法
pub trait BudgetValueMut: BudgetValue {
    /// 设置优先级
    fn set_priority(&mut self, new_p: Self::E);

    /// 设置耐久度
    fn set_durability(&mut self, new_d: Self::E);

    /// 设置质量
    fn set_quality(&mut self, new_q: Self::E);

    // TODO: 复现更多所需功能
}

/// 一个默认实现
/// * 🔬仅作测试用
pub type Budget = (f64, f64, f64);

impl BudgetValue for Budget {
    // 指定为浮点数
    type E = f64;

    fn priority(&self) -> f64 {
        self.0
    }

    fn durability(&self) -> f64 {
        self.1
    }

    fn quality(&self) -> f64 {
        self.2
    }
}
