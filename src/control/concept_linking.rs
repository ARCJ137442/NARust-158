//! NARS控制机制/概念链接
//! * 📍复合词项的「词项链模板」搭建
//! * 📍复合词项「链接到任务」的功能

use crate::{
    entity::{TLinkType, TermLinkTemplate},
    language::{CompoundTermRef, Term},
};

/// Build TermLink templates to constant components and sub-components
///
/// The compound type determines the link type; the component type determines whether to build the link.
pub fn prepare_term_link_templates(this: &Term) -> Vec<TermLinkTemplate> {
    // * 🚩创建返回值
    let mut links_to_self = Vec::new();
    match this.as_compound() {
        Some(compound) => {
            // * 🚩预备「默认类型」：自身为陈述⇒陈述，自身为复合⇒复合
            let initial_term_link_type = match this.instanceof_statement() {
                true => TLinkType::CompoundStatement,
                false => TLinkType::Compound, // default
            };
            // * 🚩建立连接：从「自身到自身」开始
            prepare_component_links(
                compound,
                &mut links_to_self,
                initial_term_link_type,
                compound.inner,
            );
            links_to_self
        }
        // * 🚩不是复合词项⇒返回空
        None => links_to_self,
    }
}

/// Collect TermLink templates into a list, go down one level except in special cases
/// * ❗重要逻辑：词项链的构造 | ❓看似构造了「从元素链接到自身」但实际上「目标」却是「元素」
fn prepare_component_links(
    whole: CompoundTermRef,
    links: &mut Vec<TermLinkTemplate>,
    term_link_type: TLinkType,
    inner: &Term,
) {
    todo!()
}
