//! 🆕抽象出「输入通道」与「输出通道」的共同特征
//! * 🎯替代[推理器](crate::nars::Reasoner)中的`removeXXXXXChannel`方法
//!   * 不违反借用规则，同时也无需判等

use crate::inference::ReasonContext;

/// 🆕统一「输入通道」「输出通道」的「通道」类型
/// * 🎯替代并整合推理器中「移除通道」的方法——标记删除法
///   * 🚩核心逻辑：标记「待删除」然后让推理器自行决定
pub trait Channel {
    /// 参数调用中「推理上下文」所需之类型
    type Context: ReasonContext;

    // /// 参数调用中「参考推理器」所需的类型
    // /// * 📌只能限制在一个，否则整个特征就不是「对象安全」的
    // /// * 🚩【2024-05-17 17:09:25】现在限制无效了：因为需要「推理上下文」作为泛型参数
    // ///
    // type Reasoner: Reasoner<Self::Context>;

    /// 🆕判断是否「应该被移除」
    /// * 🚩推理器在下次遍历通道之前，会先移除所有在此返回`true`的
    /// * 🎯替代`reasoner.removeInputChannel(this);`
    ///   * 📌动态分派的**特征对象难以判等**
    fn need_remove(&self) -> bool;
}
