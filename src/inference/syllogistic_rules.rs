//! 🎯复刻OpenNARS `nars.inference.SyllogisticRules`
//! * 📄有关「类型声明」参见[「推理上下文」](super::reason_context)

use super::ReasonContext;
use crate::{entity::*, inference::*};

/// 模拟`SyllogisticRules`
pub trait SyllogisticRules: ReasonContext {
    /// 模拟`SyllogisticRules.________`
    ///
    /// # 📄OpenNARS
    fn ________(task: &Self::Task, belief: &Self::Sentence, memory: &mut Self::Memory) {
        /* 📄OpenNARS源码：
         */
    }
}

/// 自动实现，以便添加方法
impl<T: ReasonContext> SyllogisticRules for T {}

/// TODO: 单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
