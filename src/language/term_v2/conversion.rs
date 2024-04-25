//! 与其它类型相互转换
//! * 🎯转换为「词法Narsese」以便「获取名称」

use super::*;
use anyhow::{anyhow, Result};
use narsese::{
    conversion::{
        inter_type::lexical_fold::TryFoldInto, string::impl_lexical::format_instances::FORMAT_ASCII,
    },
    lexical::Term as TermLexical,
};
use std::str::FromStr;

// 格式化所用常量
const COMPONENT_OPENER: &str = "(";
const COMPONENT_CLOSER: &str = ")";
const COMPONENT_SEPARATOR: &str = " ";

/// 词项⇒字符串
/// * 🎯用于更好地打印「词项」名称
impl Term {
    pub fn format_name(&self) -> String {
        let id = &self.identifier;
        match &*self.components {
            // 空组分
            TermComponents::Empty => id.clone(),
            // 名称 | 原子词项
            TermComponents::Named(name) => id.clone() + name,
            // 一元
            TermComponents::Unary(term) => {
                // 📄 "(-- A)"
                manipulate!(
                    String::new()
                    => {+= COMPONENT_OPENER}#
                    => {+= id}#
                    => {+= COMPONENT_SEPARATOR}#
                    => {+= &term.format_name()}#
                    => {+= COMPONENT_CLOSER}#
                )
            }
            // 二元
            TermComponents::Binary(term1, term2) => {
                // 📄 "(A --> B)"
                manipulate!(
                    String::new()
                    => {+= COMPONENT_OPENER}#
                    => {+= &term1.format_name()}#
                    => {+= COMPONENT_SEPARATOR}#
                    => {+= id}#
                    => {+= COMPONENT_SEPARATOR}#
                    => {+= &term2.format_name()}#
                    => {+= COMPONENT_CLOSER}#
                )
            }
            // 多元
            TermComponents::Multi(terms) => {
                let mut s = id.to_string() + COMPONENT_OPENER;
                let mut terms = terms.iter();
                if let Some(t) = terms.next() {
                    s += &t.format_name();
                }
                for t in terms {
                    s += COMPONENT_SEPARATOR;
                    s += &t.format_name();
                }
                s + COMPONENT_CLOSER
            }
            // 多元+索引
            TermComponents::MultiIndexed(index, terms) => {
                let mut s = id.to_string() + COMPONENT_OPENER;
                let mut terms = terms.iter();
                // 分「占位符在开头」与「占位符在后头」
                if *index == 0 {
                    s += PLACEHOLDER;
                    for term in terms {
                        s += COMPONENT_SEPARATOR;
                        s += &term.format_name();
                    }
                } else {
                    // * ⚠️【2024-04-22 13:02:41】SAFETY: 经由「像」的构造函数保证，占位符必定在界内
                    // 占位符前的词项
                    s += &terms.next().unwrap().format_name();
                    for _ in 1..*index {
                        s += COMPONENT_SEPARATOR;
                        s += &terms.next().unwrap().format_name();
                    }
                    // 占位符
                    s += COMPONENT_SEPARATOR;
                    s += PLACEHOLDER;
                    // 占位符后的词项
                    for term in terms {
                        s += COMPONENT_SEPARATOR;
                        s += &term.format_name();
                    }
                }
                s + COMPONENT_CLOSER
            }
        }
    }
}

/// 词项⇒词法Narsese
impl From<&Term> for TermLexical {
    fn from(value: &Term) -> Self {
        use TermComponents::*;
        let (id, comp) = value.id_comp();
        match (id, comp) {
            // 专用 / 集合词项 | 默认已排序
            (SET_EXT_OPERATOR | SET_INT_OPERATOR, Multi(v)) => {
                let v = v.iter().map(TermLexical::from).collect::<Vec<_>>();
                Self::new_compound(id, v)
            }
            // 专用 / 陈述
            (
                INHERITANCE_RELATION | SIMILARITY_RELATION | IMPLICATION_RELATION
                | EQUIVALENCE_RELATION,
                Binary(subj, pred),
            ) => Self::new_statement(id, subj.into(), pred.into()),
            // 通用 / 空：仅前缀
            (_, Empty) => Self::new_atom(id, ""),
            // 通用 / 具名：前缀+词项名
            (_, Named(name)) => Self::new_atom(id, name),
            // 通用 / 一元
            (_, Unary(term)) => Self::new_compound(id, vec![term.into()]),
            // 通用 / 二元
            (_, Binary(subj, pred)) => Self::new_compound(id, vec![subj.into(), pred.into()]),
            // 多元
            (_, Multi(terms)) => {
                Self::new_compound(id, terms.iter().map(TermLexical::from).collect())
            }
            // 通用 / 带索引
            (_, MultiIndexed(i, v)) => {
                // 逐个转换组分
                let mut v = v.iter().map(TermLexical::from).collect::<Vec<_>>();
                // 创建并插入「占位符」
                let placeholder = Term::new_placeholder();
                let placeholder = (&placeholder).into();
                v.insert(*i, placeholder);
                // 构造 & 返回
                Self::new_compound(id, v)
            }
        }
    }
}

/// 词法折叠 / 获取「标识符」
/// * 🎯从「词法Narsese」获取「标识符」，以便后续根据「标识符」分发逻辑
/// * 🚩对「集合」词项：将左右括弧直接拼接，作为新的、统一的「标识符」
fn get_identifier(term: &TermLexical) -> String {
    match term {
        TermLexical::Atom { prefix, .. } => prefix.clone(),
        TermLexical::Compound { connecter, .. } => connecter.clone(),
        TermLexical::Set {
            left_bracket,
            right_bracket,
            ..
        } => left_bracket.to_string() + right_bracket,
        TermLexical::Statement { copula, .. } => copula.clone(),
    }
}

/// 词法折叠 / 从「数组」中转换
/// * 🎯将「词法Narsese词项数组」转换为「内部词项数组」
/// * 📌在「无法同时`map`与`?`」时独立成函数
/// * ⚠️不允许构造空词项数组：参考NAL，不允许空集
#[inline]
fn fold_lexical_terms(terms: Vec<TermLexical>) -> Result<Vec<Term>> {
    let mut v = vec![];
    for term in terms {
        v.push(term.try_into()?);
    }
    check_folded_terms(v)
}

fn check_folded_terms(v: Vec<Term>) -> Result<Vec<Term>> {
    match v.is_empty() {
        true => Err(anyhow!("NAL不允许构造空集")),
        false => Ok(v),
    }
}

/// 词法折叠 / 从「数组」中转换成「像」
/// * 🎯将「词法Narsese词项数组」转换为「像」所需的「带索引词项数组」
#[inline]
fn fold_lexical_terms_as_image(terms: Vec<TermLexical>) -> Result<(usize, Vec<Term>)> {
    // 构造「组分」
    let mut v = vec![];
    let mut placeholder_index = 0;
    for (i, term) in terms.into_iter().enumerate() {
        let term: Term = term.try_into()?;
        // 识别「占位符位置」
        // 🆕【2024-04-21 01:12:50】不同于OpenNARS：只会留下（且位置取决于）最后一个占位符
        // 📄OpenNARS在「没找到占位符」时，会将第一个元素作为占位符，然后把「占位符索引」固定为`1`
        match term.is_placeholder() {
            true => placeholder_index = i,
            false => v.push(term),
        }
    }
    Ok((placeholder_index, check_folded_terms(v)?))
}

/// 词法折叠
impl TryFoldInto<'_, Term, anyhow::Error> for TermLexical {
    type Folder = ();

    /// 💭【2024-04-21 14:44:15】目前此中方法「相较保守」
    /// * 📌与词法Narsese严格对应（ASCII）
    /// * ✅基本保证「解析结果均保证『合法』」
    fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
        let identifier = get_identifier(&self);
        let self_str = FORMAT_ASCII.format(&self);
        // 在有限的标识符范围内匹配
        use TermLexical::*;
        let term = match (identifier.as_str(), self) {
            // 原子词项 | ⚠️虽然「单独的占位符」在OpenNARS中不合法，但在解析「像」时需要用到 //
            (WORD, Atom { name, .. }) => Term::new_word(name),
            (PLACEHOLDER, Atom { .. }) => Term::new_placeholder(),
            (VAR_INDEPENDENT, Atom { name, .. }) => Term::new_var_i(name),
            (VAR_DEPENDENT, Atom { name, .. }) => Term::new_var_d(name),
            (VAR_QUERY, Atom { name, .. }) => Term::new_var_q(name),
            // 复合词项 //
            (SET_EXT_OPERATOR, Set { terms, .. }) => Term::new_set_ext(fold_lexical_terms(terms)?),
            (SET_INT_OPERATOR, Set { terms, .. }) => Term::new_set_int(fold_lexical_terms(terms)?),
            (INTERSECTION_EXT_OPERATOR, Compound { terms, .. }) => {
                Term::new_intersect_ext(fold_lexical_terms(terms)?)
            }
            (INTERSECTION_INT_OPERATOR, Compound { terms, .. }) => {
                Term::new_intersect_int(fold_lexical_terms(terms)?)
            }
            (DIFFERENCE_EXT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
                let mut iter = terms.into_iter();
                let term1 = iter.next().unwrap().try_into()?;
                let term2 = iter.next().unwrap().try_into()?;
                Term::new_diff_ext(term1, term2)
            }
            (DIFFERENCE_INT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
                let mut iter = terms.into_iter();
                let term1 = iter.next().unwrap().try_into()?;
                let term2 = iter.next().unwrap().try_into()?;
                Term::new_diff_int(term1, term2)
            }
            (PRODUCT_OPERATOR, Compound { terms, .. }) => {
                Term::new_product(fold_lexical_terms(terms)?)
            }
            (IMAGE_EXT_OPERATOR, Compound { terms, .. }) => {
                let (i, terms) = fold_lexical_terms_as_image(terms)?;
                match i {
                    // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                    0 => Term::new_product(terms),
                    _ => Term::new_image_ext(i, terms)?,
                }
            }
            (IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
                let (i, terms) = fold_lexical_terms_as_image(terms)?;
                match i {
                    // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                    0 => Term::new_product(terms),
                    _ => Term::new_image_int(i, terms)?,
                }
            }
            (CONJUNCTION_OPERATOR, Compound { terms, .. }) => {
                Term::new_conjunction(fold_lexical_terms(terms)?)
            }
            (DISJUNCTION_OPERATOR, Compound { terms, .. }) => {
                Term::new_disjunction(fold_lexical_terms(terms)?)
            }
            (NEGATION_OPERATOR, Compound { terms, .. }) if terms.len() == 1 => {
                Term::new_negation(terms.into_iter().next().unwrap().try_into()?)
            }
            // 陈述
            (
                INHERITANCE_RELATION,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_inheritance(subject.try_fold_into(&())?, predicate.try_fold_into(&())?),
            (
                SIMILARITY_RELATION,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_similarity(subject.try_fold_into(&())?, predicate.try_fold_into(&())?),
            (
                IMPLICATION_RELATION,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_implication(subject.try_fold_into(&())?, predicate.try_fold_into(&())?),
            (
                EQUIVALENCE_RELATION,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_equivalence(subject.try_fold_into(&())?, predicate.try_fold_into(&())?),
            (
                INSTANCE_RELATION, // 派生系词/实例
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_inheritance(
                Term::new_set_ext(vec![subject.try_fold_into(&())?]),
                predicate.try_fold_into(&())?,
            ),

            (
                PROPERTY_RELATION, // 派生系词/属性
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_inheritance(
                subject.try_fold_into(&())?,
                Term::new_set_int(vec![predicate.try_fold_into(&())?]),
            ),
            (
                INSTANCE_PROPERTY_RELATION, // 派生系词/实例属性
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new_inheritance(
                Term::new_set_ext(vec![subject.try_fold_into(&())?]),
                Term::new_set_int(vec![predicate.try_fold_into(&())?]),
            ),
            // 其它情况⇒不合法
            _ => return Err(anyhow!("非法词项：{self_str:?}")),
        };
        Ok(term)
    }
    /*
    /// 💭【2024-04-21 13:40:40】目前这种方法还是「过于粗放」
    ///   * ⚠️容许系统内没有的词项类型
    ///   * ⚠️容许【即便标识符在定义内，但『组分』类型不同】的情况
    fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
        let identifier = get_identifier(&self);
        use TermLexical::*;
        let term = match (identifier.as_str(), self) {
            // 专用 / 占位符
            (PLACEHOLDER, _) => Term::new_placeholder(),
            // 专用 / 一元复合词项
            (NEGATION_OPERATOR, Compound { mut terms, .. }) => {
                // 仅在长度为1时返回成功
                if terms.len() == 1 {
                    // ! ⚠️若使用`get`会导致「重复引用」
                    let term = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                    Term::new_negation(term)
                } else {
                    return Err(anyhow!("非法的一元复合词项组分：{terms:?}"));
                }
            }
            // 专用 / 二元复合词项（有序）
            (DIFFERENCE_EXT_OPERATOR | DIFFERENCE_INT_OPERATOR, Compound { mut terms, .. }) => {
                // 仅在长度为2时返回成功
                if terms.len() == 2 {
                    // ! ⚠️若使用`get`会导致「重复引用」
                    let term2 = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                    let term1 = unsafe { terms.pop().unwrap_unchecked().try_fold_into(&())? };
                    Term::new(identifier, TermComponents::Binary(term1, term2))
                } else {
                    return Err(anyhow!("非法的二元复合词项组分：{terms:?}"));
                }
            }
            // 专用 / 无序陈述
            (
                SIMILARITY_RELATION | EQUIVALENCE_RELATION,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new(
                identifier,
                TermComponents::new_binary_unordered(
                    subject.try_fold_into(&())?,
                    predicate.try_fold_into(&())?,
                ),
            ),
            // 专用 / 无序复合词项 | 不含「词项集」（在「集合词项」中）
            (
                INTERSECTION_EXT_OPERATOR
                | INTERSECTION_INT_OPERATOR
                | CONJUNCTION_OPERATOR
                | DISJUNCTION_OPERATOR,
                Compound { terms, .. },
            ) => Term::new(
                identifier,
                // 视作「多元集合」：排序 & 去重
                TermComponents::new_multi_set(vec_from_lexical_terms(terms)?),
            ),
            // 专用 / 像
            (IMAGE_EXT_OPERATOR | IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
                // 构造「组分」
                let mut v = vec![];
                let mut placeholder_index = 0;
                for (i, term) in terms.into_iter().enumerate() {
                    let term: Term = term.try_fold_into(&())?;
                    // 识别「占位符位置」
                    // 🆕【2024-04-21 01:12:50】不同于OpenNARS：只会留下（且位置取决于）最后一个占位符
                    // 📄OpenNARS在「没找到占位符」时，会将第一个元素作为占位符，然后把「占位符索引」固定为`1`
                    match term.is_placeholder() {
                        true => placeholder_index = i,
                        false => v.push(term),
                    }
                }
                // 构造 & 返回
                Term::new(
                    identifier,
                    TermComponents::MultiIndexed(placeholder_index, v),
                )
            }
            // 通用 / 原子词项
            // * 📄词语
            // * 📄变量
            (_, Atom { name, .. }) => Term::new(identifier, TermComponents::Named(name)),
            // 通用 / 复合词项 | 默认视作有序
            // * 📄乘积
            (_, Compound { terms, .. }) => Term::new(
                identifier,
                TermComponents::Multi(vec_from_lexical_terms(terms)?),
            ),
            // 通用 / 集合词项 | 默认视作无序
            // * 📄外延集、内涵集
            (_, Set { terms, .. }) => Term::new(
                identifier,
                // 视作「多元集合」：排序 & 去重
                TermComponents::new_multi_set(vec_from_lexical_terms(terms)?),
            ),
            // 通用 / 陈述 | 默认视作有序
            // * 📄继承、蕴含
            (
                _,
                Statement {
                    subject, predicate, ..
                },
            ) => Term::new(
                identifier,
                TermComponents::Binary(
                    subject.try_fold_into(&())?,
                    predicate.try_fold_into(&())?,
                ),
            ),
            // // 其它⇒返回错误
            // ! 🚩【2024-04-21 01:38:15】已穷尽
            // _ => return Err(anyhow!("未知词项标识符：{identifier:?}")),
        };
        Ok(term)
    } */
}

/// 基于「词法折叠」实现[`TryFrom`]
impl TryFrom<TermLexical> for Term {
    type Error = anyhow::Error;

    #[inline(always)]
    fn try_from(value: TermLexical) -> Result<Self, Self::Error> {
        value.try_fold_into(&())
    }
}

/// 字符串解析路线：词法解析 ⇒ 词法折叠
/// * 🎯同时兼容[`str::parse`]与[`str::try_into`]
impl TryFrom<&str> for Term {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // 词法解析
        let lexical = FORMAT_ASCII.parse(s)?;
        // 词法转换 | ⚠️对「语句」「任务」报错
        let term = lexical.try_into_term()?;
        // 词法折叠
        let term = term.try_into()?;
        // 返回
        Ok(term)
    }
}

///  字符串解析
/// * 🎯同时兼容[`str::parse`]与[`str::try_into`]
impl FromStr for Term {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use narsese::{
        conversion::{
            inter_type::lexical_fold::TryFoldInto,
            string::impl_lexical::format_instances::FORMAT_ASCII,
        },
        lexical::Term as LexicalTerm,
        lexical_nse_term,
    };

    /// 测试 / 词法折叠
    #[test]
    fn test_lexical_fold() -> Result<()> {
        fn fold(t: LexicalTerm) -> Result<Term> {
            print!("{:?} => ", FORMAT_ASCII.format(&t));
            let term: Term = t.try_fold_into(&())?;
            println!("{:?}", term.format_name());
            Ok(term)
        }
        fold(lexical_nse_term!(<A --> B>))?;
        fold(lexical_nse_term!((&&, C, B, A, (/, A, _, B))))?;
        fold(lexical_nse_term!(<(*, {SELF}, x, y) --> ^left>))?;
        fold(lexical_nse_term!([2, 1, 0, $0, #1, ?2]))?;
        fold(lexical_nse_term!(<A <-> {A}>))?;
        fold(lexical_nse_term!(<{B} <=> B>))?;
        fold(lexical_nse_term!(<{SELF} ==> (--, [good])>))?;
        Ok(())
    }
}
