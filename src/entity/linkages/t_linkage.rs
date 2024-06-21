use super::{TLink, TLinkType};

/// T链接的一个默认实现
pub struct TLinkage<Target> {
    /// The linked Target
    /// * 📝【2024-05-30 19:39:14】final化：一切均在构造时确定，构造后不再改变
    target: Target,

    /// The type of link, one of the above
    link_type: TLinkType,

    /// The index of the component in the component list of the compound,
    /// may have up to 4 levels
    /// * 📝「概念推理」中经常用到
    /// * 🚩构造后即不可变
    index: Box<[usize]>,
}

impl<Target> TLinkage<Target> {
    /// 完全构造方法
    /// * 📄OpenNARS中仅需在子类中暴露
    /// * 🚩【2024-06-22 01:06:50】为了与「词项链模板」使用同一类型，此处避开`new`名称
    ///   * 方便使用[`TermLinkTemplate::new`]而不受歧义
    pub(crate) fn new_direct(
        target: Target,
        link_type: TLinkType,
        index: impl Into<Box<[usize]>>,
    ) -> Self {
        Self {
            target,
            link_type,
            index: index.into(),
        }
    }

    /// 🆕「目标」的别名
    pub fn will_from_self_to(&self) -> &Target {
        &self.target
    }
}

impl<Target> TLink<Target> for TLinkage<Target> {
    fn target(&self) -> &Target {
        &self.target
    }

    fn target_mut(&mut self) -> &mut Target {
        &mut self.target
    }

    fn link_type(&self) -> TLinkType {
        self.link_type
    }

    fn indexes(&self) -> &[usize] {
        &self.index
    }
}
