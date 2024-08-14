//! 作为「词项链」与「任务链」共有的特征基础存在

use crate::symbols::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// 基于枚举的「链接类型」
/// * 📌【2024-06-04 19:35:12】拨乱反正：此处的「类型名」均为「从自身向目标」视角下「目标相对自身」的类型
/// * 📄目标是自身的元素⇒COMPONENT「元素」链接
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TLinkType {
    /// From C, targeted to "SELF" C; TaskLink only
    /// * 🚩【2024-06-22 00:26:43】避嫌Rust的`Self`关键字
    SELF,
    /// From (&&, A, C), targeted to "COMPONENT" C
    Component,
    /// From C, targeted to "COMPOUND" (&&, A, C)
    Compound,
    /// From <C --> A>, targeted to "COMPONENT_STATEMENT" C
    ComponentStatement,
    /// From C, targeted to "COMPOUND_STATEMENT" <C --> A>
    CompoundStatement,
    /// From <(&&, C, B) ==> A>, targeted to "COMPONENT_CONDITION" C
    ComponentCondition,
    /// From C, targeted to "COMPOUND_CONDITION" <(&&, C, B) ==> A>
    CompoundCondition,
    /// From C, targeted to "TRANSFORM" <(*, C, B) --> A>; TaskLink only
    Transform,
}

impl TLinkType {
    /// 🆕获取「链接类型」的「排序」，即原OpenNARS中的编号
    /// * 📝【2024-06-22 00:32:57】不建议在结构体定义之处附带数值
    ///   * 📌附带的数值只能是[`isize`]类型
    ///   * ❌无法在【不增加维护成本】的同时转换到[`usize`]类型
    pub fn to_order(&self) -> usize {
        use TLinkType::*;
        match self {
            SELF => 0,
            Component => 1,
            Compound => 2,
            ComponentStatement => 3,
            CompoundStatement => 4,
            ComponentCondition => 5,
            CompoundCondition => 6,
            Transform => 8,
        }
    }

    /// 🆕判断一个「T链接类型」是否为「从复合词项链接到元素」
    pub fn is_to_component(&self) -> bool {
        use TLinkType::*;
        // 1 3 5
        matches!(self, Component | ComponentStatement | ComponentCondition)
    }

    /// 🆕判断一个「T链接类型」是否为「从元素链接到复合词项」
    pub fn is_to_compound(&self) -> bool {
        use TLinkType::*;
        // 2 4 6
        // * 🚩【2024-06-04 18:25:26】目前不包括TRANSFORM
        matches!(self, Compound | CompoundStatement | CompoundCondition)
    }

    /// 🆕从「元素→整体」变成「整体→元素」
    /// * 🚩「自元素到整体」⇒「自整体到元素」
    /// * 📌【2024-06-04 19:51:48】目前只在「元素→整体」⇒「整体→元素」的过程中调用
    /// * ✅【2024-06-22 00:38:55】此处使用「默认返回自身」兼容
    pub fn try_point_to_component(self) -> Self {
        // // * 🚩改版中只会发生在`COMPOUND`变种中
        // debug_assert!(
        //     matches!(self, Compound | CompoundStatement | CompoundCondition),
        //     "原始值 {self:?} 并非指向复合词项"
        // );
        use TLinkType::*;
        match self {
            Compound => Component,
            ComponentStatement => ComponentStatement,
            CompoundStatement => ComponentStatement,
            CompoundCondition => ComponentCondition,
            // * 🚩其它的默认逻辑：返回自身 | 这也是其所用之处的默认情况
            // ! 🤦【2024-08-05 01:44:56】血泪教训：别盲目兼容
            //   * 📝不然这「默认兼容情况」就可能有「漏网之鱼」
            _ => panic!("不支持的转换：{self:?}"),
        }
    }
}

/// 🆕任务链与词项链共有的「T链接」
/// * 🚩【2024-06-01 20:56:49】现在不再基于[`Item`]，交由后续「词项链」「任务链」「词项链模板」自由组合
pub trait TLink<Target> {
    /// 链接所指目标
    /// * ⚠️此处不能只是引用：可能会有「共享引用代理」的情况
    ///   * 🚩【2024-06-22 12:13:37】目前仿照[`crate::global::RC`]的签名，改为「可解引用的类型」
    ///   * ✅此举可实现「引用」和「共享引用代理」的兼容
    fn target<'r, 's: 'r>(&'s self) -> impl Deref<Target = Target> + 'r;

    /// 目标的可变引用
    /// * ⚠️此处不能只是引用：可能会有「共享引用代理」的情况
    ///   * 🚩【2024-06-22 12:13:37】目前仿照[`crate::global::RC`]的签名，改为「可解引用的类型」
    ///   * ✅此举可实现「引用」和「共享引用代理」的兼容
    /// * 🎯在「任务链」中需要
    ///   * 📄推理上下文中仅靠「任务链」获取「当前任务」
    fn target_mut<'r, 's: 'r>(&'s mut self) -> impl DerefMut<Target = Target> + 'r;

    /// 链接类型
    /// * 📌【2024-06-22 00:41:22】[`TLinkType`]实现了[`Copy`]，所以直接传出所有权
    fn link_type(&self) -> TLinkType;

    /// 获取链接的「索引」，指向具体词项的位置
    /// * 🎯表示「链接起点」与「链接目标」词项的位置关系
    /// * 📌创建后不再改变（只读）
    fn indexes(&self) -> &[usize];

    /// Set the key of the link
    /// * 📝原`setKey`就是「根据现有信息计算出key，并最终给自身key赋值」的功能
    /// * 🚩【2024-05-30 19:06:30】现在不再有副作用，仅返回key让调用方自行决定
    /// * 📌原`setKey()`要变成`this.key = generateKey(this.type, this.index)`
    /// * 🚩目前不再使用继承机制，而是在各个实现中使用特化的函数
    #[doc(alias = "generate_key")]
    fn generate_key_base(link_type: TLinkType, indexes: &[usize]) -> String {
        // * 🚩先添加左右括弧，分「向元素」和「向整体」表示
        // * 📌格式：自身 - 目标 | "_"即「元素」
        // * 📝 向元素: 整体 "@(【索引】)_" 元素
        // * 📝 向整体: 元素 "_@(【索引】)" 整体
        let [at1, at2] = match link_type.is_to_component() {
            true => TO_COMPONENT,
            false => TO_COMPOUND,
        };
        let mut inner = format!("T{}", link_type.to_order());
        for index in indexes {
            inner += "-";
            inner += &(index + 1).to_string();
        }
        format!("{at1}{inner}{at2}")
    }

    /// Get one index by level
    fn get_index(&self, index: usize) -> Option<&usize> {
        self.indexes().get(index)
    }

    /// 快速假定性获取索引
    /// * 🎯假定在界内；若在界外，则panic
    fn index(&self, index: usize) -> usize {
        self.indexes()[index]
    }
}
