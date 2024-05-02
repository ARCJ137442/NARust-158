//! * 🚩核心逻辑：一个前提，多个派生，多方聚合
//!   * 前提：通过实现[`EvidenceReal`]得到「基本操作」
//!   * 派生：通过实现各类`XXXFunctions`得到「派生操作」
//!   * 聚合：通过统一的「自动实现」得到「所有操作汇聚于一体」的静态功能增强（真值函数@数值）
//!     * 📝Rust允许「在外部调用『看似没有实现派生操作的结构』时，允许使用『自动实现了的派生操作』」
//! * 🕒最后更新：【2024-05-02 16:15:14】

use narsese::api::EvidentNumber;
/// 🆕【前提】抽象的「证据数值」特征
/// * 🚩【2024-05-02 16:05:04】搬迁自[`crate::entity::BudgetValue`]
/// * 🚩扩展自「证据值」，并（可）实验性地、敏捷开发地为之添加方法
/// * 🚩【2024-05-02 16:03:35】目前用于实验性添加「必要内容」
///   * 📌这些内容后续将被纳入[Narsese.rs的`api`模块](narsese::api)
/// * 💭【2024-05-02 00:46:02】亦有可能替代OpenNARS的`nars.inference.UtilityFunctions`
/// * 🚩【2024-05-02 17:48:30】现在全部抛弃基于「不可变引用」的运算
///   * ⚠️混合「传可变引用」和「直接传值」的代码将过于冗杂（并且造成接口不统一）
///   * 📌在实现了[`Copy`]之后，将值的复制看作是「随处可用」的
pub trait EvidenceReal:
// * 🚩【2024-05-02 18:33:19】将`Ord`作为在[`EvidentNumber`]之上的「附加要求」之一：需要在「预算值合并」使用「取最大」方法
    EvidentNumber + Copy + Ord /* + TryFrom<Float, Error = Self::TryFromError> */
{
    // * 🚩【2024-05-02 18:47:49】现在随着「无需支持`to_float`浮点转换」不再要求（↑`TryFrom`同理）
    // /// * 📌此处对[`Error`](std::fmt::Error)的需求仅仅在于[`Result::unwrap`]需要`Error: Debug`
    // /// * 🎯【2024-05-02 12:17:19】引入以兼容[`TryFrom`]的[`try_from`](TryFrom::try_from)
    // type TryFromError: std::error::Error;

    // /// 转换为浮点数
    // /// * 🚩使用「全局浮点数类型」
    // /// * 🎯用于【预算数值与普通浮点数之间】【不同的预算数值之间】互相转换
    // ///   * 📄「几何均值」在最后需要「n次开根」
    // ///   * 📄`w2c`函数需要从值域 $[0, 1]$ 扩展到 $[0, +\infty)$
    // fn to_float(&self) -> Float;

    /// 设置值
    /// * 📝【2024-05-02 17:50:19】亦可使用[`Clone`]从其它地方（就地）拷贝
    /// * 🚩【2024-05-02 17:50:33】目前随「普遍传值」采取「直接赋值」的方法
    #[inline(always)]
    fn set(&mut self, new_value: Self) {
        // self.clone_from(new_value)
        *self = new_value;
    }
}
