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
                compound,
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
    current: CompoundTermRef,
) {
    /* 第一层元素 */
    for (i, t1) in current.components.iter().enumerate() {
        // * 🚩「常量」词项⇒直接链接 | 构建「元素→自身」的「到复合词项」类型
        if t1.is_constant() {
            links.push(TermLinkTemplate::new_template(
                t1.clone(),
                term_link_type,
                vec![i],
            ));
            // * 📝【2024-05-15 18:21:25】案例笔记 概念="<(&&,A,B) ==> D>"：
            // * 📄self="<(&&,A,B) ==> D>" ~> "(&&,A,B)" [i=0]
            // * @ 4=COMPOUND_STATEMENT "At C, point to <C --> A>"
            // * 📄self="(&&,A,B)" ~> "A" [i=0]
            // * @ 6=COMPOUND_CONDITION "At C, point to <(&&, C, B) ==> A>"
            // * 📄self="(&&,A,B)" ~> "B" [i=1]
            // * @ 6=COMPOUND_CONDITION "At C, point to <(&&, C, B) ==> A>"
            // * 📄self="<(&&,A,B) ==> D>" ~> "D" [i=1]
            // * @ 4=COMPOUND_STATEMENT "At C, point to <C --> A>"
            // * 📄self="(&&,A,B)" ~> "A" [i=0]
            // * @ 2=COMPOUND "At C, point to (&&, A, C)"
            // * 📄self="(&&,A,B)" ~> "B" [i=1]
            // * @ 2=COMPOUND "At C, point to (&&, A, C)"
        }
        // * 🚩条件类链接⇒递归
        // * 📌自身和索引必须先是「蕴含の主词」或「等价」，如 <# ==> C> 或 <# <=> #>
        // * 💥【2024-06-18 21:03:35】此处将「等价」从「复合条件」除籍，理由如下：
        // * * 「等价」可以通过类似「继承⇄相似」的方式产生「蕴含」
        // * * 许多推理规则均在「复合条件」链接类型中假设「链接目标」为「蕴含」词项
        let is_conditional_compound = whole.instanceof_implication() && i == 0;
        // * 🚩然后「内部词项」必须是「合取」或「否定」
        let is_conditional_component = t1.instanceof_conjunction() || t1.instanceof_negation();
        let is_conditional = is_conditional_compound && is_conditional_component;
        if is_conditional {
            if let Some(t1) = t1.as_compound() {
                // * 📝递归深入，将作为「入口」的「自身向自身建立链接」缩小到「组分」区域
                // * 🚩改变「默认类型」为「复合条件」
                prepare_component_links(t1, links, TLinkType::CompoundCondition, t1);
            }
        }
        // * 🚩其它情况⇒若元素为复合词项，再度深入
        else if let Some(t1) = t1.as_compound() {
            /* 第二层元素 */
            for (j, t2) in t1.components.iter().enumerate() {
                // * 🚩直接处理 @ 第二层
                if t2.is_constant() {
                    let transform_t1 = t1.instanceof_product() || t1.instanceof_image();
                    if transform_t1 {
                        // * 🚩NAL-4「转换」相关 | 构建「复合→复合」的「转换」类型（仍然到复合词项）
                        let indexes = match term_link_type {
                            // * 📝若背景的「链接类型」已经是「复合条件」⇒已经深入了一层，并且一定在「主项」位置
                            TLinkType::CompoundCondition => vec![0, i, j],
                            // * 📝否则就还是第二层
                            _ => vec![i, j],
                        };
                        links.push(TermLinkTemplate::new_template(
                            t2.clone(),
                            TLinkType::Transform,
                            indexes,
                        ));
                    } else {
                        // * 🚩非「转换」相关：直接按类型添加 | 构建「元素→自身」的「到复合词项」类型
                        links.push(TermLinkTemplate::new_template(
                            t2.clone(),
                            term_link_type,
                            vec![i, j],
                        ));
                    }
                }
                /* 第三层元素 */
                // * 🚩直接处理 @ 第三层
                if let Some(t2) =
                    t2.as_compound_and(|t2| t2.instanceof_product() || t2.instanceof_image())
                {
                    // * 🚩NAL-4「转换」相关 | 构建「复合→复合」的「转换」类型（仍然到复合词项）
                    for (k, t3) in t2.components.iter().enumerate() {
                        if t3.is_constant() {
                            let indexes = match term_link_type {
                                // * 📝此处若是「复合条件」即为最深第四层
                                TLinkType::CompoundCondition => vec![0, i, j, k],
                                // * 📝否则仅第三层
                                _ => vec![i, j, k],
                            };
                            links.push(TermLinkTemplate::new_template(
                                t3.clone(),
                                TLinkType::Transform,
                                indexes,
                            ));
                        }
                    }
                }
            }
        }
    }
}

// TODO: 单元测试
#[cfg(test)]
mod tests {}
