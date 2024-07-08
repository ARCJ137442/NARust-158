//! 与其它类型相互转换
//! * 🎯转换为「词法Narsese」以便「获取名称」

use super::structs::*;
use crate::io::symbols::*;
use anyhow::{anyhow, Result};
use nar_dev_utils::*;
use narsese::{
    api::{FormatTo, GetCapacity},
    conversion::inter_type::lexical_fold::TryFoldInto,
    lexical::Term as TermLexical,
};
use std::str::FromStr;

/// 词项⇒字符串
/// * 🎯用于更好地打印「词项」名称
/// * 🎯用于从「词法Narsese」中解析
///   * 考虑「变量语义」
impl Term {
    pub fn format_name(&self) -> String {
        // 格式化所用常量
        const OPENER: &str = "(";
        const CLOSER: &str = ")";
        const SEPARATOR: &str = " ";

        use narsese::api::TermCapacity::*;
        use TermComponents::*;
        let id = &self.identifier;
        match &self.components {
            // 空组分
            Empty => id.clone(),
            // 名称 | 原子词项
            Word(name) => id.clone() + name,
            // 名称 | 变量词项
            Variable(n) => id.clone() + &n.to_string(),
            Compound(terms) => {
                match self.get_capacity() {
                    // 一元
                    Unary => {
                        // 📄 "(-- A)"
                        manipulate!(
                            String::new()
                            => {+= OPENER}#
                            => {+= id}#
                            => {+= SEPARATOR}#
                            => {+= &terms[0].format_name()}#
                            => {+= CLOSER}#
                        )
                    }
                    // 二元
                    BinaryVec | BinarySet => {
                        // 📄 "(A --> B)"
                        manipulate!(
                            String::new()
                            => {+= OPENER}#
                            => {+= &terms[0].format_name()}#
                            => {+= SEPARATOR}#
                            => {+= id}#
                            => {+= SEPARATOR}#
                            => {+= &terms[1].format_name()}#
                            => {+= CLOSER}#
                        )
                    }
                    // 多元
                    Vec | Set => {
                        let mut s = id.to_string() + OPENER;
                        let mut terms = terms.iter();
                        if let Some(t) = terms.next() {
                            s += &t.format_name();
                        }
                        for t in terms {
                            s += SEPARATOR;
                            s += &t.format_name();
                        }
                        s + CLOSER
                    }
                    Atom => unreachable!("复合词项只可能是「一元」「二元」或「多元」"),
                }
            }
        }
    }

    /// 尝试从「词法Narsese」转换
    /// * 💭【2024-04-21 14:44:15】目前此中方法「相较保守」
    /// * 📌与词法Narsese基本对应（ASCII）
    /// * ✅基本保证「解析结果均保证『合法』」
    /// * 🚩【2024-06-13 18:39:33】现在是「词法折叠」使用本处实现
    /// * ⚠️在「词法折叠」的过程中，即开始「变量匿名化」
    ///   * 📌【2024-07-02 00:40:39】需要保证「格式化」的是个「整体」：变量只在「整体」范围内有意义
    #[inline(always)]
    pub fn from_lexical(lexical: TermLexical) -> Result<Self> {
        fold_term(lexical, &mut vec![])
    }

    /// 从「内部Narsese」转换为「词法Narsese」
    /// * 🚩基本无损转换（无需考虑失败情况）
    pub fn to_lexical(&self) -> TermLexical {
        use TermComponents::*;
        type LTerm = TermLexical;
        let (id, comp) = self.id_comp();
        match (id, comp) {
            // 专用 / 集合词项 | 默认已排序
            (SET_EXT_OPERATOR, Compound(v)) => {
                let v = v.iter().map(Self::to_lexical).collect::<Vec<_>>();
                LTerm::new_set(SET_EXT_OPENER, v, SET_EXT_CLOSER)
            }
            (SET_INT_OPERATOR, Compound(v)) => {
                let v = v.iter().map(Self::to_lexical).collect::<Vec<_>>();
                LTerm::new_set(SET_INT_OPENER, v, SET_INT_CLOSER)
            }
            //  陈述
            (
                INHERITANCE_RELATION | SIMILARITY_RELATION | IMPLICATION_RELATION
                | EQUIVALENCE_RELATION,
                Compound(terms),
            ) if terms.len() == 2 => {
                LTerm::new_statement(id, (&terms[0]).into(), (&terms[1]).into())
            }
            // 通用 / 空：仅前缀
            (_, Empty) => LTerm::new_atom(id, ""),
            // 通用 / 具名：前缀+词项名
            (_, Word(name)) => LTerm::new_atom(id, name),
            // 通用 / 变量：前缀+变量编号
            (_, Variable(num)) => LTerm::new_atom(id, num.to_string()),
            // 通用 / 多元
            (_, Compound(terms)) => {
                LTerm::new_compound(id, terms.iter().map(Self::to_lexical).collect())
            }
        }
    }

    /// 转换为显示呈现上的ASCII格式
    /// * 📌对标OpenNARS的默认呈现
    /// * ⚠️【2024-07-02 00:52:54】目前需要「词法Narsese」作为中间格式，可能会有性能损失
    pub fn to_display_ascii(&self) -> String {
        use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
        self.to_lexical().format_to(&FORMAT_ASCII)
    }

    /// 尝试从「方言」转换
    /// * 🎯支持「方言解析」
    /// * 📌【2024-05-15 02:33:13】目前仍只有「从字符串到词项」这一种形式
    /// * 🆕附加功能，与核心「数据算法」「推理控制」无关
    #[inline(always)]
    #[cfg(feature = "dialect_parser")]
    pub fn from_dialect(input: &str) -> Result<Self> {
        use super::super::dialect::parse_term;
        parse_term(input)
    }
}

// * 🚩此处的「变量词项」一开始就应该是个数值，从「具名变量」变为「数字变量」
/// 词项⇒词法Narsese
impl From<&Term> for TermLexical {
    fn from(term: &Term) -> Self {
        term.to_lexical()
    }
}

impl From<TermComponents> for Vec<Term> {
    /// 将「词项组分」转换为「可变数组<词项>」
    /// * 🚩原子词项⇒空数组
    /// * 🚩复合词项⇒其内所有词项构成的数组
    fn from(value: TermComponents) -> Self {
        use TermComponents::*;
        match value {
            Empty | Word(..) | Variable(..) => vec![],
            Compound(terms) => terms.into(),
        }
    }
}

impl From<TermComponents> for Box<[Term]> {
    /// 将「词项组分」转换为「定长数组<词项>」
    /// * 🚩原子词项⇒空数组
    /// * 🚩复合词项⇒其内所有词项构成的数组
    /// * ℹ️与上述对[`Vec`]的转换不同：此处直接使用`Box::new([])`构造空数组
    fn from(value: TermComponents) -> Self {
        use TermComponents::*;
        match value {
            Empty | Word(..) | Variable(..) => Box::new([]),
            Compound(terms) => terms,
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

/// 根部折叠（带「变量编号化」逻辑）
/// * 🚩接受初始化后的数组
/// * ℹ️可能被递归调用
fn fold_term(term: TermLexical, var_id_map: &mut Vec<String>) -> Result<Term> {
    /// 更新并返回一个「变量词项」，根据传入的「变量id映射」将原「变量名」映射到「变量id」
    #[inline]
    fn update_var(
        original_name: String,
        var_id_map: &mut Vec<String>,
        new_var_from_id: fn(usize) -> Term, // * 📝不用特意引用
    ) -> Term {
        match var_id_map
            .iter()
            .position(|stored_name| &original_name == stored_name)
        {
            // * 🚩id从1开始
            Some(existed) => new_var_from_id(existed + 1),
            // * 🚩新名称
            None => {
                var_id_map.push(original_name);
                new_var_from_id(var_id_map.len())
            }
        }
    }
    // 正式开始
    let identifier = get_identifier(&term);
    // 在有限的标识符范围内匹配
    use TermLexical::*;
    let term = match (identifier.as_str(), term) {
        // 原子词项 | ⚠️虽然「单独的占位符」在OpenNARS中不合法，但在解析「像」时需要用到 //
        (WORD, Atom { name, .. }) => Term::new_word(name),
        (PLACEHOLDER, Atom { .. }) => Term::new_placeholder(),
        (VAR_INDEPENDENT, Atom { name, .. }) => update_var(name, var_id_map, Term::new_var_i),
        (VAR_DEPENDENT, Atom { name, .. }) => update_var(name, var_id_map, Term::new_var_d),
        (VAR_QUERY, Atom { name, .. }) => update_var(name, var_id_map, Term::new_var_q),
        // 复合词项 //
        (SET_EXT_OPERATOR, Set { terms, .. }) => {
            Term::new_set_ext(fold_lexical_terms(terms, var_id_map)?)
        }
        (SET_INT_OPERATOR, Set { terms, .. }) => {
            Term::new_set_int(fold_lexical_terms(terms, var_id_map)?)
        }
        (INTERSECTION_EXT_OPERATOR, Compound { terms, .. }) => {
            Term::new_intersection_ext(fold_lexical_terms(terms, var_id_map)?)
        }
        (INTERSECTION_INT_OPERATOR, Compound { terms, .. }) => {
            Term::new_intersection_int(fold_lexical_terms(terms, var_id_map)?)
        }
        (DIFFERENCE_EXT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
            let mut iter = terms.into_iter();
            let term1 = fold_term(iter.next().unwrap(), var_id_map)?;
            let term2 = fold_term(iter.next().unwrap(), var_id_map)?;
            Term::new_diff_ext(term1, term2)
        }
        (DIFFERENCE_INT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
            let mut iter = terms.into_iter();
            let term1 = fold_term(iter.next().unwrap(), var_id_map)?;
            let term2 = fold_term(iter.next().unwrap(), var_id_map)?;
            Term::new_diff_int(term1, term2)
        }
        (PRODUCT_OPERATOR, Compound { terms, .. }) => {
            Term::new_product(fold_lexical_terms(terms, var_id_map)?)
        }
        (IMAGE_EXT_OPERATOR, Compound { terms, .. }) => {
            // ! ⚠️现在解析出作为「像之内容」的「词项序列」包含「占位符」作为内容
            let (i, terms) = fold_lexical_terms_as_image(terms, var_id_map)?;
            match i {
                // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                0 => Term::new_product(terms),
                _ => Term::new_image_ext(terms)?,
            }
        }
        (IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
            // ! ⚠️现在解析出作为「像之内容」的「词项序列」包含「占位符」作为内容
            let (i, terms) = fold_lexical_terms_as_image(terms, var_id_map)?;
            match i {
                // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                0 => Term::new_product(terms),
                _ => Term::new_image_int(terms)?,
            }
        }
        (CONJUNCTION_OPERATOR, Compound { terms, .. }) => {
            Term::new_conjunction(fold_lexical_terms(terms, var_id_map)?)
        }
        (DISJUNCTION_OPERATOR, Compound { terms, .. }) => {
            Term::new_disjunction(fold_lexical_terms(terms, var_id_map)?)
        }
        (NEGATION_OPERATOR, Compound { terms, .. }) if terms.len() == 1 => {
            let inner = fold_term(terms.into_iter().next().unwrap(), var_id_map)?;
            Term::new_negation(inner)
        }
        // 陈述
        (
            INHERITANCE_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_inheritance(
            fold_term(*subject, var_id_map)?,
            fold_term(*predicate, var_id_map)?,
        ),
        (
            SIMILARITY_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_similarity(
            fold_term(*subject, var_id_map)?,
            fold_term(*predicate, var_id_map)?,
        ),
        (
            IMPLICATION_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_implication(
            fold_term(*subject, var_id_map)?,
            fold_term(*predicate, var_id_map)?,
        ),
        (
            EQUIVALENCE_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_equivalence(
            fold_term(*subject, var_id_map)?,
            fold_term(*predicate, var_id_map)?,
        ),
        (
            INSTANCE_RELATION, // 派生系词/实例
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_inheritance(
            Term::new_set_ext(vec![fold_term(*subject, var_id_map)?]),
            fold_term(*predicate, var_id_map)?,
        ),
        (
            PROPERTY_RELATION, // 派生系词/属性
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_inheritance(
            fold_term(*subject, var_id_map)?,
            Term::new_set_int(vec![fold_term(*predicate, var_id_map)?]),
        ),
        (
            INSTANCE_PROPERTY_RELATION, // 派生系词/实例属性
            Statement {
                subject, predicate, ..
            },
        ) => Term::new_inheritance(
            Term::new_set_ext(vec![fold_term(*subject, var_id_map)?]),
            Term::new_set_int(vec![fold_term(*predicate, var_id_map)?]),
        ),
        // 其它情况⇒不合法
        (identifier, this) => return Err(anyhow!("标识符为「{identifier}」的非法词项：{this:?}")),
    };
    Ok(term)
}

/// 词法折叠 / 从「数组」中转换
/// * 🎯将「词法Narsese词项数组」转换为「内部词项数组」
/// * 📌在「无法同时`map`与`?`」时独立成函数
/// * ⚠️不允许构造空词项数组：参考NAL，不允许空集
#[inline]
fn fold_lexical_terms(terms: Vec<TermLexical>, var_id_map: &mut Vec<String>) -> Result<Vec<Term>> {
    let mut v = vec![];
    for term in terms {
        v.push(fold_term(term, var_id_map)?);
    }
    check_folded_terms(v)
}

/// 检查折叠好了的词项表
/// * 🚩【2024-06-14 00:13:29】目前仅检查「是否为空集」
fn check_folded_terms(v: Vec<Term>) -> Result<Vec<Term>> {
    match v.is_empty() {
        true => Err(anyhow!("词法折叠错误：NAL不允许构造空集")),
        false => Ok(v),
    }
}

/// 词法折叠 / 从「数组」中转换成「像」
/// * 🎯将「词法Narsese词项数组」转换为「像」所需的「带索引词项数组」
#[inline]
fn fold_lexical_terms_as_image(
    terms: Vec<TermLexical>,
    var_id_map: &mut Vec<String>,
) -> Result<(usize, Vec<Term>)> {
    // 构造「组分」
    let mut v = vec![];
    let mut placeholder_index = 0;
    for (i, term) in terms.into_iter().enumerate() {
        let term: Term = fold_term(term, var_id_map)?;
        // 识别「占位符位置」
        // 🆕【2024-04-21 01:12:50】不同于OpenNARS：只会留下（且位置取决于）最后一个占位符
        // 📄OpenNARS在「没找到占位符」时，会将第一个元素作为占位符，然后把「占位符索引」固定为`1`
        match term.is_placeholder() {
            true => {
                placeholder_index = i;
                if i > 0 {
                    // * 🚩占位符不能是第一个⇒否则作为「乘积」提交（不包含占位符）
                    v.push(term);
                }
            }
            // * 🚩现在除了「占位符在第一个」（乘积）的情形，其它均将「占位符」算入在「元素」中
            false => v.push(term),
        }
    }
    Ok((placeholder_index, check_folded_terms(v)?))
}

/// 词法折叠
impl TryFoldInto<'_, Term, anyhow::Error> for TermLexical {
    type Folder = ();

    fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
        Term::from_lexical(self)
    }
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
/// * 📌使用标准OpenNARS ASCII语法
impl TryFrom<&str> for Term {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
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
    use crate::{ok, util::AResult};
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
    fn test_lexical_fold() -> AResult {
        fn fold(t: LexicalTerm) -> Result<Term> {
            print!("{:?} => ", FORMAT_ASCII.format(&t));
            let term: Term = t.try_fold_into(&())?;
            println!("{:?}", term.format_name());
            Ok(term)
        }
        fold(lexical_nse_term!(<A --> B>))?;
        fold(lexical_nse_term!((&&, C, B, A, (/, A, _, B))))?;
        // fold(lexical_nse_term!(<(*, {SELF}, x, y) --> ^left>))?; // ! ⚠️【2024-04-25 10:02:20】现在对「操作符」不再支持
        fold(lexical_nse_term!([2, 1, 0, $0, #1, ?2]))?;
        fold(lexical_nse_term!(<A <-> {A}>))?;
        fold(lexical_nse_term!(<{B} <=> B>))?;
        fold(lexical_nse_term!(<{SELF} ==> (--, [good])>))?;
        ok!()
    }

    /// 测试 / 变量重编号
    /// * 📄nse  `<(&&,<(*,{$1},{$2},$d)-->方向>,<(*,{$1},$c)-->格点状态>,<(*,{$2},无缺陷)-->格点状态>)==><(*,$d,$c,{$1},{$2})-->[同色连空]>>.%1.00;0.99%`
    ///   * 🕒【2024-07-02 00:32:46】
    ///   * 预期：`<(&&,<(*,{$1},{$2},$3)-->方向>,<(*,{$1},$4)-->格点状态>,<(*,{$2},无缺陷)-->格点状态>)==><(*,$3,$4,{$1},{$2})-->[同色连空]>>. %1.0;0.99%`
    ///   * 当前：`<(&&,<(*,{$1},无缺陷)-->格点状态>,<(*,{$1},$2)-->格点状态>,<(*,{$1},{$2},$3)-->方向>)==><(*,$1,$2,{$3},{$4})-->[同色连空]>>.%1.0000;0.9900%`
    ///   * 预期の映射：
    ///     * `$1` => `$1`
    ///     * `$2` => `$2`
    ///     * `$d` => `$3`
    ///     * `$c` => `$4`
    ///   * 当前の映射：
    ///     * `$1` => `$1`
    ///     * `$2` => `$2`、`$1`@「无缺陷」
    ///     * `$d` => `$3`、`$1`@同色连空
    ///     * `$c` => `$2`
    /// * ✅【2024-07-02 01:06:12】现在成功：至少是唯一映射了
    ///     * `$1` => `$1`
    ///     * `$2` => `$2`
    ///     * `$d` => `$4`
    ///     * `$c` => `$3`
    #[test]
    fn test_var_map() -> AResult {
        // 词法Narsese展示
        let lexical = lexical_nse_term!(<(&&,<(*,{$1},{$2},$d)-->方向>,<(*,{$1},$c)-->格点状态>,<(*,{$2},无缺陷)-->格点状态>)==><(*,$d,$c,{$1},{$2})-->[同色连空]>>);
        println!("{}", FORMAT_ASCII.format(&lexical));

        // 词法折叠
        let term1 = Term::from_lexical(lexical.clone())?;
        let term1_s = term1.to_display_ascii();
        println!("{term1_s}");

        // 内部折叠方法
        let mut var_map = vec![];
        let term2 = fold_term(lexical.clone(), &mut var_map)?;
        let term2_s = term2.to_display_ascii();
        println!("{term2_s}");
        assert_eq!(term1_s, term2_s); // 两种转换之后，字符串形式应该相等

        // 对比：映射表
        println!("{:?}", var_map);
        for (var_i, original_name) in var_map.iter().enumerate() {
            println!("{original_name} => {}", var_i + 1);
        }
        let expected = [("1", 1), ("2", 2), ("d", 3), ("c", 4)];
        for (original_name, var_i) in expected.iter() {
            // 断言映射表相等
            assert_eq!(var_map[*var_i - 1], *original_name);
        }
        ok!()
    }
}
