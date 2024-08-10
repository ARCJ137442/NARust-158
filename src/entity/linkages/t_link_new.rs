use nar_dev_utils::{manipulate, unwrap_or_return};

use super::StatementPosition;
use crate::{
    io::symbols::CONJUNCTION_OPERATOR,
    language::{StatementRef, Term},
};

/// 根部的「链接位置」
/// * 📌存储「链接到自身」「元素→整体」或「整体→元素」
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TLinkPosition {
    /// From C, targeted to "SELF" C; TaskLink only
    /// * 🚩【2024-06-22 00:26:43】避嫌Rust的`Self`关键字
    SELF,
    Compound(Vec<TLinkIndex>),
    Component(Vec<TLinkIndex>),
}

/// 其中一个链接索引
/// * 📍设计上一个个组成链（列表），以表示某个词项在另一词项中的位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TLinkIndex {
    /// From (&&, A, C), targeted to "COMPONENT" C
    /// From C, targeted to "COMPOUND" (&&, A, C)
    Compound(usize),
    /// From <C --> A>, targeted to "COMPONENT_STATEMENT" C
    /// From C, targeted to "COMPOUND_STATEMENT" <C --> A>
    Statement(StatementPosition),
    /// From <(&&, C, B) ==> A>, targeted to "COMPONENT_CONDITION" C
    /// From C, targeted to "COMPOUND_CONDITION" <(&&, C, B) ==> A>
    Condition(StatementPosition, usize), // 一次跨越两层
    /// From C, targeted to "TRANSFORM" <(*, C, B) --> A>; TaskLink only
    Transform(StatementPosition, usize), // 一次跨越两层
}

impl TLinkIndex {
    /// 一个索引占多少深度
    pub fn depth(&self) -> usize {
        use TLinkIndex::*;
        match self {
            Compound(..) | Statement(..) => 1,
            Condition(..) | Transform(..) => 2,
        }
    }

    /// 将索引转换为纯非负整数数组
    /// * ⚠️抹去「复合词项/陈述」的信息
    pub fn indexes(&self) -> Box<[usize]> {
        use TLinkIndex::*;
        match self {
            Compound(i) => Box::new([*i]),
            Statement(p) => Box::new([*p as usize]),
            Condition(p, i) | Transform(p, i) => Box::new([*p as usize, *i]),
        }
    }

    /// 将一系列的索引转换为纯非负整数数组
    /// * ❌【2024-08-11 00:28:07】纯迭代器方法不可行：`Box<[T]>`无法真正`into_iter`
    pub fn into_indexes<'a>(iter: impl IntoIterator<Item = &'a TLinkIndex>) -> Vec<usize> {
        let mut indexes = Vec::new();
        for index in iter {
            indexes.extend(index.indexes().iter());
        }
        indexes
    }

    /// 在某个词项上指向某个目标
    /// * 🚩词项的结构需要严格满足链接所述之类型，否则返回空
    pub fn index_on<'t>(&self, term: &'t Term) -> Option<&'t Term> {
        use TLinkIndex::*;
        // 首先是复合词项
        todo!()
    }
}

/// 临时构建的「链接模板」
/// * 📌只包含
///   * 索引
///   * 目标
pub type TL = (Vec<TLinkIndex>, Term);

/// Build TermLink templates to constant components and sub-components
///
/// The compound type determines the link type; the component type determines whether to build the link.
pub fn prepare_term_link_templates(this: &Term) -> Vec<TL> {
    let mut links_to_self = vec![];
    _prepare_term_link_templates(this, &[], &mut links_to_self);
    links_to_self
}

/// 用于递归的内部函数
/// * ⚠️【2024-08-11 00:23:38】已知与OpenNARS机制不同的地方在于：
///   * 会不分「复合条件」「转换规则」地多构造链接（最大深度=4）
///   * 如下例子：要不下边的取不到所有三个，要不上边的多`chirping`和`flying`
///   * 📄"<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>"
///   * 📄"<(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>"
fn _prepare_term_link_templates(current: &Term, base: &[TLinkIndex], out: &mut Vec<TL>) {
    use StatementPosition::*;
    const MAX_LINK_DEPTH: usize = 4;

    // 添加链接的闭包
    let mut add_link = |index, sub_term: &Term, recursive| {
        // 制作新索引
        let indexes = manipulate! {
            base.to_vec()
            => .push(index)
        };
        // 计算并检验深度
        let current_depth = indexes.iter().map(TLinkIndex::depth).sum::<usize>();
        if current_depth > MAX_LINK_DEPTH && indexes.len() < MAX_LINK_DEPTH {
            return;
        }
        // 递归深入
        if recursive {
            let new_base = indexes.as_slice();
            _prepare_term_link_templates(sub_term, new_base, out);
        }
        // 仅添加「常量词项」
        if sub_term.is_constant() {
            // 添加链接
            out.push((indexes, sub_term.clone()))
        }
    };
    // 首先是复合词项
    let compound = unwrap_or_return!(?current.as_compound());
    // 陈述
    if let Some([sub, pre]) = compound
        .as_statement()
        .filter(|s| s.instanceof_statement())
        .as_ref()
        .map(StatementRef::sub_pre)
    {
        for (term, pos) in [(sub, Subject), (pre, Predicate)] {
            let mut recursive = true;

            // 转换句
            // * 📝只有继承句中的「像」或「乘积」能拥有「转换」索引
            if let Some(product_or_image) =
                current.as_compound_type(CONJUNCTION_OPERATOR).filter(|_| {
                    current.instanceof_inheritance()
                        && (term.instanceof_product() || term.instanceof_image())
                })
            {
                // 深入添加子项
                for (i, component) in product_or_image.components.iter().enumerate() {
                    add_link(TLinkIndex::Transform(pos, i), component, recursive);
                }
                recursive = false; // 与「普通陈述」不互斥，但只在一处递归
            }

            // 条件句
            // * 📝只有蕴含句中前项的「合取」能拥有「转换」索引
            if let Some(conjunction) = term
                .as_compound_type(CONJUNCTION_OPERATOR)
                .filter(|_| pos == Subject && current.instanceof_implication())
            {
                // 深入添加子项
                for (i, component) in conjunction.components.iter().enumerate() {
                    add_link(TLinkIndex::Condition(pos, i), component, recursive);
                }
                recursive = false; // 与「普通陈述」不互斥，但只在一处递归
            }

            // 其它
            add_link(TLinkIndex::Statement(pos), term, recursive);
        }
        return;
    }
    // 普通复合词项 | 与「陈述」互斥
    for (i, component) in compound.components.iter().enumerate() {
        add_link(TLinkIndex::Compound(i), component, true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ok, test_term as term, util::AResult};

    #[test]
    fn test_links() -> AResult {
        fn test(term: Term) {
            let links = prepare_term_link_templates(&term);
            println!("{term}");
            for (tag, target) in links {
                println!("~> {target}\t{tag:?}\t{:?}", TLinkIndex::into_indexes(&tag));
            }
        }
        test(term!(
            "<(&&, <(*, A, B) --> R>, C, <(/, R, _, B) <-> $1>) ==> <S --> P>>"
        ));
        test(term!("(&&, <(*, A, B) --> R>, C, <(/, R, _, B) <-> $1>)"));

        test(term!("(*, A, B)"));
        test(term!("{A, B, C, D}"));
        test(term!("(/, R, _, A)"));
        test(term!("(/, R, A, _, B)"));
        test(term!("<A --> B>"));
        test(term!("<(&&, A, B) ==> C>"));
        test(term!("<<$1 --> key> ==> <{lock1} --> (/, open, $1, _)>>"));
        test(term!(
            "<(&&,<#1 --> lock>,<#1 --> (/,open,$2,_)>) ==> <$2 --> key>>"
        ));
        test(term!(
            "<(&&,<robin --> [chirping]>,<robin --> [flying]>) ==> <robin --> bird>>"
        ));
        ok!()
    }
}
