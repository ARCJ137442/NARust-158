//! 管理有关Narsese词项「词法折叠」的功能
//! * 🎯实现「词法Narsese→内部Narsese」的转换
//!   * ⚠️存在「语义无效」的情况
//!     * 📄在NAL之外的词项类型
//!     * 📄空集、重言式等「语义无效」词项

use super::super::base::*;
use crate::symbols::*;
use anyhow::{anyhow, Result};
use narsese::lexical::Term as TermLexical;

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

/// 词法折叠的上下文对象
/// * 🎯统一存储如「变量id映射」的临时状态
#[derive(Debug, Clone)]
struct FoldContext {
    /// 有关「变量id⇒词项名称」的映射
    var_id_map: Vec<String>,
}

impl FoldContext {
    /// 创建一个新的上下文
    fn new() -> Self {
        Self { var_id_map: vec![] }
    }
}

/// 「词法折叠」的总入口
#[inline]
pub fn lexical_fold(term: TermLexical) -> Result<Term> {
    let mut context = FoldContext::new();
    fold_term(term, &mut context)
}

/// 「词法折叠」的递归入口
/// * 🚩大体调用流程：`conversion` => `term_making` => `construct`
///   * 【折叠】时【制作】词项，最终才【构造】
///   * 🚧【2024-09-06 17:43:36】有待实装
/// * 📌带「变量编号化」逻辑
fn fold_term(term: TermLexical, context: &mut FoldContext) -> Result<Term> {
    // TODO: 理清「折叠时简化」与「make」的 区别/差异
    // ? ❓简化的时机
    // ? ❓是否要「边解析边简化」「内部元素解析简化后再到此处」
    // TODO: 简化其中的「make」相关选项
    // * 📄何时对「内部词项」排序

    /// 更新并返回一个「变量词项」，根据传入的「变量id映射」将原「变量名」映射到「变量id」
    #[inline]
    fn update_var(
        var_type: impl Into<String>,
        original_name: String,
        context: &mut FoldContext,
    ) -> Term {
        match context
            .var_id_map
            .iter()
            .position(|stored_name| &original_name == stored_name)
        {
            // * 🚩id从1开始
            Some(existed) => Term::from_var_similar(var_type, existed + 1),
            // * 🚩新名称
            None => {
                context.var_id_map.push(original_name);
                Term::from_var_similar(var_type, context.var_id_map.len())
            }
        }
    }

    macro_rules! make_error {
        () => {
            if cfg!(test) {
                anyhow::anyhow!("词项无效 @ {}:{}", file!(), line!())
            } else {
                anyhow::anyhow!("词项无效")
            }
        };
    }

    // 正式开始
    let identifier = get_identifier(&term);
    // 在有限的标识符范围内匹配
    use TermLexical::*;
    let term = match (identifier.as_str(), term) {
        // 原子词项 | ⚠️虽然「单独的占位符」在OpenNARS中不合法，但在解析「像」时需要用到 //
        (WORD, Atom { name, .. }) => Term::make_word(name),
        (PLACEHOLDER, Atom { .. }) => Term::make_placeholder(),
        (VAR_INDEPENDENT, Atom { name, .. }) => update_var(VAR_INDEPENDENT, name, context),
        (VAR_DEPENDENT, Atom { name, .. }) => update_var(VAR_DEPENDENT, name, context),
        (VAR_QUERY, Atom { name, .. }) => update_var(VAR_QUERY, name, context),
        // 复合词项 //
        (SET_EXT_OPERATOR, Set { terms, .. }) => {
            Term::make_set_ext_arg(fold_inner_lexical_vec(terms, context)?).ok_or(make_error!())?
        }
        (SET_INT_OPERATOR, Set { terms, .. }) => {
            Term::make_set_int_arg(fold_inner_lexical_vec(terms, context)?).ok_or(make_error!())?
        }
        (INTERSECTION_EXT_OPERATOR, Compound { terms, .. }) => {
            Term::make_intersection_ext_arg(fold_inner_lexical_vec(terms, context)?)
                .ok_or(make_error!())?
        }
        (INTERSECTION_INT_OPERATOR, Compound { terms, .. }) => {
            Term::make_intersection_int_arg(fold_inner_lexical_vec(terms, context)?)
                .ok_or(make_error!())?
        }
        (DIFFERENCE_EXT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
            let mut iter = terms.into_iter();
            let term1 = fold_inner_lexical(iter.next().unwrap(), context)?;
            let term2 = fold_inner_lexical(iter.next().unwrap(), context)?;
            Term::make_difference_ext(term1, term2).ok_or(make_error!())?
        }
        (DIFFERENCE_INT_OPERATOR, Compound { terms, .. }) if terms.len() == 2 => {
            let mut iter = terms.into_iter();
            let term1 = fold_inner_lexical(iter.next().unwrap(), context)?;
            let term2 = fold_inner_lexical(iter.next().unwrap(), context)?;
            Term::make_difference_int(term1, term2).ok_or(make_error!())?
        }
        (PRODUCT_OPERATOR, Compound { terms, .. }) => {
            Term::make_product_arg(fold_inner_lexical_vec(terms, context)?).ok_or(make_error!())?
        }
        (IMAGE_EXT_OPERATOR, Compound { terms, .. }) => {
            // ! ⚠️现在解析出作为「像之内容」的「词项序列」包含「占位符」作为内容
            let (i, terms) = fold_lexical_terms_as_image(terms, context)?;
            match i {
                // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                0 => Term::make_product_arg(terms).ok_or(make_error!())?,
                _ => Term::make_image_ext_vec(terms).ok_or(make_error!())?,
            }
        }
        (IMAGE_INT_OPERATOR, Compound { terms, .. }) => {
            // ! ⚠️现在解析出作为「像之内容」的「词项序列」包含「占位符」作为内容
            let (i, terms) = fold_lexical_terms_as_image(terms, context)?;
            match i {
                // 占位符在首位⇒视作「乘积」 | 📝NAL-4中保留「第0位」作「关系」词项
                0 => Term::make_product_arg(terms).ok_or(make_error!())?,
                _ => Term::make_image_int_vec(terms).ok_or(make_error!())?,
            }
        }
        (CONJUNCTION_OPERATOR, Compound { terms, .. }) => {
            Term::make_conjunction_arg(fold_inner_lexical_vec(terms, context)?)
                .ok_or(make_error!())?
        }
        (DISJUNCTION_OPERATOR, Compound { terms, .. }) => {
            Term::make_disjunction_arg(fold_inner_lexical_vec(terms, context)?)
                .ok_or(make_error!())?
        }
        (NEGATION_OPERATOR, Compound { terms, .. }) if terms.len() == 1 => {
            // TODO: 提取形如「数组中『判断指定数量并取出数组』」的语义 `fn extract_term_vec<const N: usize>(terms: Vec<Term>) -> Result<[Term; N]>`
            // * 💡使用「占位符」作为「数组初始化」的占位符
            let inner = fold_inner_lexical(terms.into_iter().next().unwrap(), context)?;
            Term::make_negation(inner).ok_or(make_error!())?
        }
        (SEQUENCE_OPERATOR, Compound { terms, .. }) => {
            Term::make_sequence(fold_inner_lexical_vec(terms, context)?).ok_or(make_error!())?
        }
        // 陈述
        (
            INHERITANCE_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_inheritance(
            fold_inner_lexical(*subject, context)?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            SIMILARITY_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_similarity(
            fold_inner_lexical(*subject, context)?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            IMPLICATION_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_implication(
            fold_inner_lexical(*subject, context)?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            TEMPORAL_IMPLICATION_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_temporal_implication(
            fold_inner_lexical(*subject, context)?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            EQUIVALENCE_RELATION,
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_equivalence(
            fold_inner_lexical(*subject, context)?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            INSTANCE_RELATION, // 派生系词/实例
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_inheritance(
            Term::make_set_ext_arg(vec![fold_inner_lexical(*subject, context)?])
                .ok_or(make_error!())?,
            fold_inner_lexical(*predicate, context)?,
        )
        .ok_or(make_error!())?,
        (
            PROPERTY_RELATION, // 派生系词/属性
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_inheritance(
            fold_inner_lexical(*subject, context)?,
            Term::make_set_int_arg(vec![fold_inner_lexical(*predicate, context)?])
                .ok_or(make_error!())?,
        )
        .ok_or(make_error!())?,
        (
            INSTANCE_PROPERTY_RELATION, // 派生系词/实例属性
            Statement {
                subject, predicate, ..
            },
        ) => Term::make_inheritance(
            Term::make_set_ext_arg(vec![fold_inner_lexical(*subject, context)?])
                .ok_or(make_error!())?,
            Term::make_set_int_arg(vec![fold_inner_lexical(*predicate, context)?])
                .ok_or(make_error!())?,
        )
        .ok_or(make_error!())?,
        // 其它情况⇒不合法
        (identifier, this) => return Err(anyhow!("标识符为「{identifier}」的非法词项：{this:?}")),
    };
    Ok(term)
}

/// 词法折叠/单个转换
/// * ⚠️拒绝呈递占位符：不允许「像占位符」在除了「外延像/内涵像」外的词项中出现
#[inline]
fn fold_inner_lexical(term: TermLexical, context: &mut FoldContext) -> Result<Term> {
    // * 🚩正常转换
    let term = fold_term(term, context)?;
    // * 🚩拦截解析出的「占位符」词项
    if term.is_placeholder() {
        return Err(anyhow!("词法折叠错误：占位符仅能直属于 外延像/内涵像 词项"));
    }
    // * 🚩正常返回
    Ok(term)
}

/// 词法折叠 / 从「数组」中转换
/// * 🎯将「词法Narsese词项数组」转换为「内部词项数组」
///   * 📄用于复合词项内部元素的解析
///   * ℹ️对于「外延像/内涵像」采用特殊方法
/// * 📌在「无法同时`map`与`?`」时独立成函数
/// * ⚠️不允许构造空词项数组：参考NAL，不允许空集
/// * ❌【2024-07-08 23:20:02】现不允许在其中解析出「占位符」词项
///   * 🎯提早避免「像占位符溢出」情形
#[inline]
fn fold_inner_lexical_vec(terms: Vec<TermLexical>, context: &mut FoldContext) -> Result<Vec<Term>> {
    let mut v = vec![];
    for term in terms {
        v.push(fold_inner_lexical(term, context)?);
    }
    check_folded_terms(v)
}

/// 检查折叠好了的词项表
/// * 🚩【2024-06-14 00:13:29】目前仅检查「是否为空集」
#[inline]
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
    context: &mut FoldContext,
) -> Result<(usize, Vec<Term>)> {
    // 构造「组分」
    let mut v = vec![];
    let mut placeholder_index = 0;
    for (i, term) in terms.into_iter().enumerate() {
        let term: Term = fold_term(term, context)?;
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

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, util::AResult};
    use nar_dev_utils::macro_once;
    use narsese::{
        conversion::{
            inter_type::lexical_fold::TryFoldInto,
            string::impl_lexical::format_instances::FORMAT_ASCII,
        },
        lexical::Term as LexicalTerm,
        lexical_nse_term as l_term,
    };

    /// 测试 / 词法折叠
    #[test]
    fn test_lexical_fold() -> AResult {
        fn test(t: LexicalTerm) -> Result<Term> {
            print!("{:?} => ", FORMAT_ASCII.format(&t));
            // 三种解析路径
            let term_1 = Term::try_from(t.clone())?;
            let term_2 = t.clone().try_fold_into(&())?;
            let term_3 = Term::from_lexical(t)?;
            // 判断路径等价性
            assert_eq!(term_1, term_2);
            assert_eq!(term_1, term_3);
            assert_eq!(term_2, term_3);
            // 打印
            let term = term_1;
            println!("{:?}", term.format_name());
            Ok(term)
        }
        macro_once! {
            // * 🚩模式：词项字符串
            macro test($($term:literal)*) {
                $(
                    test(l_term!($term))?;
                )*
            }
            "<A --> B>"
            "(&&, C, B, A, (/, A, _, B))"
            // "<(*, {SELF}, x, y) --> ^left>" // ! ⚠️【2024-04-25 10:02:20】现在对「操作符」不再支持
            "[2, 1, 0, $0, #1, ?2]"
            "<A <-> {B}>" // ! 原先的「类重言式」`<A <-> {A}>`是无效的
            "<{A} <=> B>" // ! 原先的「类重言式」`<{B} <=> B>`是无效的
            "<{SELF} ==> (--, [good])>"
        }
        ok!()
    }

    /// 测试 / 词法折叠/失败情况
    /// * ⚠️仅考虑词法折叠失败，不考虑解析失败
    #[test]
    fn test_lexical_fold_err() -> AResult {
        fn test(t: LexicalTerm) -> AResult {
            let t_s = FORMAT_ASCII.format(&t);
            let e = Term::try_from(t.clone()).expect_err(&format!("非法词项{t_s:?}异常通过解析"));
            println!("{t_s:?} => {e}");
            ok!()
        }
        macro_once! {
            // * 🚩模式：词项字符串
            macro test($($term:literal)*) {
                $(
                    test(l_term!($term))?;
                )*
            }
            // * 📄非法标识符
            // * 🚩【2024-04-25 10:02:20】现在对「操作符」不再支持
            "^operator" // ^operator
            "<(*, {SELF}, x, y) --> ^left>" // ^left
            "<X =/> Y>" // =/>
            "<X =|> Y>" // =|>
            "<X </> Y>" // </>
            "+123" // +123
            "(&/, 1, 2, 3)" // &/
            "(&|, 3, 2, 1)" // &|
            // * 📄词项数目不对
            "(-, A, B, C)"
            "(-, A)"
            "(--, A, B)"
            // * 📄空集
            // * 📄溢出的占位符
            "{_}"
            "{A, B, _}"
            "[_]"
            "[A, B, _]"
            "<A --> _>"
            "<A <-> _>"
            "<A ==> _>"
            "<A <=> _>"
            "<_ --> _>"
            "<_ <-> _>"
            "<_ ==> _>"
            "<_ <=> _>"
            "(&, _, A, B)"
            "(-, _, B)"
            "(-, A, _)"
            "(--, _)"
            "(&&, (*, [A, B, _]), A, B)"
        }
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
        let lexical = l_term!(<(&&,<(*,{$1},{$2},$d)-->方向>,<(*,{$1},$c)-->格点状态>,<(*,{$2},无缺陷)-->格点状态>)==><(*,$d,$c,{$1},{$2})-->[同色连空]>>);
        println!("{}", FORMAT_ASCII.format(&lexical));

        // 词法折叠
        let term1 = Term::from_lexical(lexical.clone())?;
        let term1_s = term1.format_ascii();
        println!("{term1_s}");

        // 内部折叠方法
        let mut context = FoldContext::new();
        let term2 = fold_term(lexical.clone(), &mut context)?;
        let term2_s = term2.format_ascii();
        println!("{term2_s}");
        assert_eq!(term1_s, term2_s); // 两种转换之后，字符串形式应该相等

        // 对比：映射表
        println!("{:?}", context);
        for (var_i, original_name) in context.var_id_map.iter().enumerate() {
            println!("{original_name} => {}", var_i + 1);
        }
        let expected = [("1", 1), ("2", 2), ("d", 3), ("c", 4)];
        for (original_name, var_i) in expected.iter() {
            // 断言映射表相等
            assert_eq!(context.var_id_map[*var_i - 1], *original_name);
        }
        ok!()
    }
}
