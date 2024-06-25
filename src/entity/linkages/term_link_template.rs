// * 📝【2024-05-15 18:37:01】实际运行中的案例（复合词项の词项链模板）：
// * 🔬复现方法：仅输入"<(&&,A,B) ==> D>."
// * ⚠️其中的内容并不完整：只列出一些有代表性的示例
// * 📄【概念】"D"
// *   <~ "<(&&,A,B) ==> D>" i=[1] # 4=COMPOUND_STATEMENT " _@(T4-2) <(&&,A,B) ==> D>"
// * 📄【概念】"(&&,A,B)"
// *   ~> "A"                i=[0] # 2=COMPOUND           " @(T1-1)_ A"
// *   ~> "B"                i=[1] # 2=COMPOUND           " @(T1-2)_ B"
// *   <~ "<(&&,A,B) ==> D>" i=[0] # 4=COMPOUND_STATEMENT " _@(T4-1) <(&&,A,B) ==> D>"
// * 📄【概念】"<(&&,A,B) ==> D>"
// *   ~> "(&&,A,B)" i=[0]   # 4=COMPOUND_STATEMENT " @(T3-1)_ (&&,A,B)"
// *   ~> "A"        i=[0,0] # 6=COMPOUND_CONDITION " @(T5-1-1)_ A"
// *   ~> "B"        i=[0,1] # 6=COMPOUND_CONDITION " @(T5-1-2)_ B"
// *   ~> "D"        i=[1]   # 4=COMPOUND_STATEMENT " @(T3-2)_ D"
// *   ~T> null      i=null  # 0=SELF               " _@(T0) <(&&,A,B) ==> D>. %1.00;0.90%"

use super::{TLinkType, TLinkage};
use crate::language::Term;

/// 「词项链模板」就是【目标为词项】的「T链接实现」
/// * ⚠️但为避免误导
pub type TermLinkTemplate = TLinkage<Term>;

/// 构造「词项链模板」
impl TermLinkTemplate {
    /// 构建新的「词项链模板」
    /// * 🚩此中的索引会根据类型动态调整，并且会限制所传入的类型
    ///   * 📌COMPOUND系列：复制原数组
    ///   * 📌COMPOUND_CONDITION：头部添加`0`
    ///   * 📌TRANSFORM：复制原数组
    ///
    /// # Panics
    ///
    /// ! 需要在传入前检查「链接类型」是否为「到复合词项」或者「转换」
    pub fn new_template(target: Term, link_type: TLinkType, index: impl Into<Vec<usize>>) -> Self {
        Self::new_direct(
            target,
            link_type,
            Self::generate_template_indexes(link_type, index),
        )
    }

    fn generate_template_indexes(
        link_type: TLinkType,
        indexes: impl Into<Vec<usize>>,
    ) -> Box<[usize]> {
        // * 🚩假定此处是「COMPOUND」系列或「TRANSFORM」类型——链接到复合词项
        debug_assert!(
            link_type.is_to_compound() || link_type == TLinkType::Transform,
            "链接类型 {link_type:?} 并非链接到复合词项"
        );
        let mut index = indexes.into();
        // * 🚩原数组为「复合条件」⇒头部添加`0`
        if link_type == TLinkType::CompoundCondition {
            index.insert(0, 0);
        }
        // * 🚩默认：复制原数组
        index.into_boxed_slice()
    }
}
