//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * ❓【2024-04-20 22:00:44】「统一结构体+用『可选字段』实现多态」的方法，会导致「性能臃肿」问题
//!   * ❗此举需要提前考虑「所有类型词项的所有功能」，并且要做到最大程度兼容
//!   * 📌即便使用「作为枚举的专用字段」也会因为「要适应某种复合词项类型」而导致让步
//!     * 而这种「只会在某个类型上产生让步」的方法，会导致「本该耦合而未耦合」的情形
//!     * 这种「看似通用，实则仍需『专用情况专用对待』」的方法，不利于后续维护
//!   * ❓【2024-04-20 23:53:15】或许也可行：是否可以`match (self.identifier, &*self.components)`
//! * 🚩【2024-04-20 22:05:09】目前将此方案搁置
//!   * ⇒尝试探索「直接基于『枚举Narsese』」的方法

use crate::io::symbols::*;

/// 作为「结构」的词项
/// * 🚩更多通过「复合」而非「抽象特征-具体实现」复用代码
///   * 📍【2024-04-20 21:13:20】目前只需实现OpenNARS 1.5.8的东西
///
///  ! ⚠️【2024-04-20 21:47:08】暂不实现「变量 < 原子 < 复合」的逻辑
///
/// # 📄OpenNARS
///
/// Term is the basic component of Narsese, and the object of processing in NARS.
/// A Term may have an associated Concept containing relations with other Terms.
/// It is not linked in the Term, because a Concept may be forgot while the Term exists. Multiple objects may represent the same Term.
///
/// ## 作为特征的「实现」
///
/// ### Cloneable => [`Clone`]
///
/// Make a new Term with the same name.
///
/// ### equals => [`Eq`]
///
/// Equal terms have identical name, though not necessarily the same reference.
///
/// ### hashCode => [`Hash`]
///
/// Produce a hash code for the term
///
/// ### compareTo => [`Ord`]
///
/// Orders among terms: variable < atomic < compound
///
/// ### toString => [`Display`]
///
/// The same as getName by default, used in display only.
///
/// @return The name of the term as a String
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Term {
    /// 标识符
    /// * 🎯决定词项的「类型」
    /// * 🚩使用不同词项类型独有的「标识符」
    ///   * 📄原子词项⇒原子词项前缀
    ///   * 📄复合词项⇒复合词项连接词
    ///   * 📄陈述⇒陈述系词
    /// * ❌【2024-04-21 00:57:39】不能使用「静态字串」固定
    ///   * ⚠️需要针对「用户输入」作一定妥协
    ///     * 此刻通过「词法折叠」等途径获得的「词项」就不一定是「静态引用」了
    ///   * 📌即便标识符的类型尽可能「固定」（就那么几种）
    identifier: String,

    /// 组分
    /// * 🎯表示「词项包含词项」的功能
    /// * 🚩通过单一的「复合组分」实现「组合」功能
    /// * 🚩此处加上[`Box`]，便不会造成「循环包含」
    components: Box<TermComponents>,
}

/// 复合词项组分
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TermComponents {
    /// 不包含任何组分
    /// * 📄占位符
    Empty,

    /// 仅包含一个字符串作为「名称」
    /// * 📄词语，变量
    Named(String),

    /// 单一组分
    /// * 📄否定
    Unary(Term),

    /// 双重组分（有序）
    /// * 📄外延差、内涵差
    /// * 📄继承、蕴含
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄相似、等价
    Binary(Term, Term),

    /// 多重组分
    /// * 📄乘积
    /// * 🚩通过「构造时自动去重并排序」实现「集合无序性」
    ///   * 📄外延集、内涵集
    ///   * 📄外延交、内涵交
    ///   * 📄合取、析取
    Multi(Vec<Term>),

    /// 多重组分（有序）+索引
    /// * 📄外延像、内涵像
    /// * ❓【2024-04-20 21:57:35】日后需要通过「像」使用时，会造成「像-MultiIndexed」绑定
    ///   * ⚡那时候若使用「断言」是否会导致不稳定
    ///   * ❓若不使用「断言」而是静默失败，是否会增加排查难度
    ///   * ❓若不使用「断言」而是发出警告，那是否会导致性能问题
    /// * 🚩可行的解决方案：`match (self.identifier, self.components) { ('/', MultiIndexed(i, v))}`
    MultiIndexed(usize, Vec<Term>),
}

impl Term {
    /// 构造函数
    /// * ⚠️有限性：仅限在「内部」使用，不希望外部以此构造出「不符范围」的词项
    pub(super) fn new(identifier: impl Into<String>, components: TermComponents) -> Self {
        Self {
            identifier: identifier.into(),
            components: Box::new(components),
        }
    }

    // 原子词项 //

    /// NAL-1 / 词语
    pub fn new_word(name: &str) -> Self {
        Self::new(WORD, TermComponents::Named(name.to_string()))
    }

    /// NAL-4 / 占位符
    /// * 📌【2024-04-21 00:36:27】需要一个「占位符」词项，以便和「词法Narsese」打交道
    /// * 🚩仅使用「占位符标识符+空组分」表示
    pub fn new_placeholder() -> Self {
        Self::new(PLACEHOLDER, TermComponents::Empty)
    }

    /// NAL-6 / 自变量
    pub fn new_var_i(name: &str) -> Self {
        Self::new(VAR_INDEPENDENT, TermComponents::Named(name.to_string()))
    }

    /// NAL-6 / 因变量
    pub fn new_var_d(name: &str) -> Self {
        Self::new(VAR_DEPENDENT, TermComponents::Named(name.to_string()))
    }

    /// NAL-6 / 查询变量
    pub fn new_var_q(name: &str) -> Self {
        Self::new(VAR_QUERY, TermComponents::Named(name.to_string()))
    }

    // 复合词项 //

    // TODO: 更多词项

    /// NAL-5 / 否定
    pub fn new_negation(term: Term) -> Self {
        Self::new(NEGATION_OPERATOR, TermComponents::Unary(term))
    }
}

/// 有关「属性」的方法集
impl Term {
    /// 只读的「标识符」属性
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// 只读的「组分」属性
    pub fn components(&self) -> &TermComponents {
        &self.components
    }

    /// 判断其是否为「占位符」
    /// * 🎯【2024-04-21 01:04:17】在「词法折叠」中首次使用
    pub fn is_placeholder(&self) -> bool {
        self.identifier == PLACEHOLDER
    }

    /// 快捷获取「标识符-组分」二元组
    /// * 🎯用于很多地方的「类型匹配」
    pub fn id_comp(&self) -> (&str, &TermComponents) {
        (&self.identifier, &*self.components)
    }

    /// 快捷获取「标识符-组分」二元组，并提供可变机会
    /// * 🚩【2024-04-21 00:59:20】现在正常返回其两重可变引用
    /// * 📝【2024-04-21 00:58:58】当「标识符」为「静态字串」时，不能对其内部的`&str`属性进行修改
    ///   * 📌使用`&mut &str`会遇到生命周期问题
    ///   * 📌实际上「修改类型」本身亦不常用
    pub fn id_comp_mut(&mut self) -> (&mut str, &mut TermComponents) {
        (&mut self.identifier, &mut *self.components)
    }
}

/// 与其它类型相互转换
/// * 🎯转换为「词法Narsese」以便「获取名称」
mod conversion {
    use super::*;
    use anyhow::{anyhow, Result};
    use nar_dev_utils::manipulate;
    use narsese::{
        conversion::inter_type::lexical_fold::TryFoldInto, lexical::Term as TermLexical,
    };

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
                TermComponents::Unary(term) => format!("({id} {})", term.format_name()),
                // 二元
                TermComponents::Binary(term1, term2) => {
                    format!("({} {id} {})", term1.format_name(), term2.format_name())
                }
                // 多元
                TermComponents::Multi(terms) => {
                    let mut s = id.to_string() + "(";
                    let mut terms = terms.iter();
                    if let Some(t) = terms.next() {
                        s += &t.format_name();
                    }
                    for t in terms {
                        s += " ";
                        s += &t.format_name();
                    }
                    s + ")"
                }
                // 多元+索引
                TermComponents::MultiIndexed(index, terms) => {
                    let mut s = id.to_string() + "(";
                    for (i, t) in terms.iter().enumerate() {
                        if i == *index {
                            if i > 0 {
                                s += " ";
                            }
                            s += PLACEHOLDER;
                        }
                        if i > 0 {
                            s += " ";
                        }
                        s += &t.format_name();
                    }
                    s + ")"
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
    #[inline]
    fn vec_from_lexical_terms(terms: Vec<TermLexical>) -> Result<Vec<Term>> {
        let mut v = vec![];
        for term in terms {
            v.push(term.try_fold_into(&())?);
        }
        Ok(v)
    }

    /// 词法折叠
    impl TryFoldInto<'_, Term, anyhow::Error> for TermLexical {
        type Folder = ();
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
                ) => {
                    // 排序：构造临时数组
                    let [term1, term2] = manipulate!(
                        [subject.try_fold_into(&())?, predicate.try_fold_into(&())?]
                        => .sort()
                    );
                    // 构造
                    Term::new(identifier, TermComponents::Binary(term1, term2))
                }
                // 专用 / 无序复合词项 | 不含「词项集」（在「集合词项」中）
                (
                    INTERSECTION_EXT_OPERATOR
                    | INTERSECTION_INT_OPERATOR
                    | CONJUNCTION_OPERATOR
                    | DISJUNCTION_OPERATOR,
                    Compound { terms, .. },
                ) => Term::new(
                    identifier,
                    // 自动排序
                    // 🆕不同于OpenNARS：此处不会进行「唯一化」处理
                    TermComponents::Multi(manipulate!(
                        vec_from_lexical_terms(terms)?
                        => .sort()
                    )),
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
                    // 自动排序
                    // 🆕不同于OpenNARS：此处不会进行「唯一化」处理
                    TermComponents::Multi(manipulate!(
                        vec_from_lexical_terms(terms)?
                        => .sort()
                    )),
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
        }
    }
}

/// 单元测试
#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use narsese::{
        conversion::{
            inter_type::lexical_fold::TryFoldInto,
            string::impl_lexical::format_instances::FORMAT_ASCII,
        },
        lexical::Term as LexicalTerm,
        lexical_nse_term,
    };

    #[test]
    fn test_term() {
        let im_ext = Term::new(
            IMAGE_EXT_OPERATOR,
            TermComponents::MultiIndexed(1, vec![Term::new_word("word")]),
        );
        fn detect(term: &Term) {
            use TermComponents::*;
            match term.id_comp() {
                (WORD, Named(name)) => {
                    println!("word with {name:?}");
                }
                (IMAGE_EXT_OPERATOR, MultiIndexed(i, v)) => {
                    println!("ext_image '/' with {i}");
                    println!("<components>");
                    for term in v {
                        detect(term);
                    }
                    println!("</components>");
                }
                _ => (),
            }
        }
        detect(&im_ext);
    }

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
        fold(lexical_nse_term!(<{SELF} ==> (-- [good])>))?;
        Ok(())
    }
}
