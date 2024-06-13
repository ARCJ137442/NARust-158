//! NARust的「方言解析器」
//! * ⚠️此文件对NARS实现（数据算法、推理控制）并无影响
//! * 💡最初启发自「Narsese呈现」中简单的「名称+符号+括号」语法
//! * 🎯表征并解析NARust数据结构

use crate::io::symbols::*;
use anyhow::{Ok, Result};
use nar_dev_utils::list;
use narsese::{
    api::NarseseOptions,
    conversion::inter_type::lexical_fold::TryFoldInto,
    lexical::{Budget, Narsese, Punctuation, Stamp, Term, Truth},
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

type MidParseResult = NarseseOptions<Budget, Term, Punctuation, Stamp, Truth>;

#[derive(Parser)] // ! ↓ 必须从项目根目录开始
#[grammar = "src/language/term_impl/_dialect.pest"]
pub struct DialectParser;

/// 使用[`pest`]将输入的「NARust方言」转换为「词法Narsese」
/// 以NARust的语法解析出Narsese
pub fn parse_lexical(input: &str) -> Result<Narsese> {
    // 语法解析
    let pair = DialectParser::parse(Rule::narsese, input)?.next().unwrap();

    // 语法折叠
    let folded = fold_pest(pair)?;

    // 返回
    Ok(folded)
}

/// 从「词法Narsese词项」转换为内部词项
pub fn parse_term(input: &str) -> Result<super::Term> {
    // 语法解析
    let pair = DialectParser::parse(Rule::narsese, input)?.next().unwrap();

    // 语法折叠
    let folded = fold_pest(pair)?.try_into_term()?;

    // 词法折叠
    let term = folded.try_fold_into(&())?;

    // 返回
    Ok(term)
}

/// 将[`pest`]解析出的[`Pair`]辅助折叠到「词法Narsese」中
fn fold_pest(pest_parsed: Pair<Rule>) -> Result<Narsese> {
    let mut mid_result = MidParseResult {
        budget: None,
        term: None,
        punctuation: None,
        stamp: None,
        truth: None,
    };
    fold_pest_procedural(pest_parsed, &mut mid_result)?;
    match mid_result.fold() {
        Some(narsese) => Ok(narsese),
        None => Err(anyhow::anyhow!("无效的中间结果")),
    }
}

/// 过程式折叠[`pest`]词法值
/// * 🎯向「中间解析结果」填充元素，而无需考虑元素的顺序与返回值类型
fn fold_pest_procedural(pair: Pair<Rule>, result: &mut MidParseResult) -> Result<()> {
    match pair.as_rule() {
        // Narsese：转发 | 📝语法文件中前缀`_`的，若为纯内容则自动忽略，若内部有元素则自动提取
        // Rule::narsese => fold_pest_procedural(pair.into_inner().next().unwrap(), result),
        // 时间戳 / 标点 ⇒ 直接插入
        Rule::punctuation => result.punctuation = Some(pair.as_str().into()),
        Rule::stamp => result.stamp = Some(pair.as_str().into()),
        // 真值 ⇒ 解析 ~ 插入
        Rule::truth => result.truth = Some(fold_pest_truth(pair)?),
        // 语句⇒所有内部元素递归 | 安装「词项」「标点」「时间戳」「真值」
        Rule::sentence => {
            for pair in pair.into_inner() {
                fold_pest_procedural(pair, result)?;
            }
        }
        // 预算⇒尝试解析并填充预算
        Rule::budget => result.budget = Some(fold_pest_budget(pair)?),
        // 任务⇒所有内部元素递归 | 安装「预算值」「语句」
        Rule::task => {
            for pair in pair.into_inner() {
                fold_pest_procedural(pair, result)?;
            }
        }
        // 词项⇒提取其中的元素 | 安装 原子 / 复合 / 陈述 | ✅pest自动解包
        // Rule::term => fold_pest_procedural(pair.into_inner().next().unwrap(), result),
        Rule::atom | Rule::compound_unary | Rule::compound_binary | Rule::compound_multi => {
            let folded = fold_pest_term(pair)?;
            let term = reform_term(folded);
            result.term = Some(term);
        }
        // 仅出现在内部解析中的不可达规则
        _ => unreachable!("仅出现在内部解析的不可达规则！{:?} {pair}", pair.as_rule()),
    }
    Ok(())
}

/// 折叠[`pest`]真值
#[inline]
fn fold_pest_truth(pair: Pair<Rule>) -> Result<Truth> {
    let mut v = Truth::new();
    for pair_value_str in pair.into_inner() {
        v.push(pair_value_str.as_str().to_string());
    }
    Ok(v)
}

/// 折叠[`pest`]预算值
#[inline]
fn fold_pest_budget(pair: Pair<Rule>) -> Result<Budget> {
    let mut v = Budget::new();
    for pair_value_str in pair.into_inner() {
        v.push(pair_value_str.as_str().to_string());
    }
    Ok(v)
}

/// 折叠[`pest`]词项
/// * 🎯用于「复合词项」内部词项的解析
/// * 📌原子、复合、陈述均可
fn fold_pest_term(pair: Pair<Rule>) -> Result<Term> {
    // 根据规则分派
    match pair.as_rule() {
        Rule::atom => fold_pest_atom(pair),
        Rule::compound_unary => fold_pest_compound_unary(pair),
        Rule::compound_binary => fold_pest_compound_binary(pair),
        Rule::compound_multi => fold_pest_compound_multi(pair),
        _ => unreachable!("词项只有可能是原子与复合 | {pair}"),
    }
}

/// 折叠[`pest`]原子词项
#[inline]
fn fold_pest_atom(pair: Pair<Rule>) -> Result<Term> {
    let mut prefix = String::new();
    let mut name = String::new();
    for pair in pair.into_inner() {
        let pair_str = pair.as_str();
        match pair.as_rule() {
            // 符号⇒前缀
            Rule::symbol_normal | Rule::symbol_raw_value => prefix.push_str(pair_str),
            // 名称⇒按「下划线前缀」分别处理
            // * 🎯占位符
            Rule::name_normal | Rule::name_raw_value => {
                let mut chars = pair_str.chars();
                for c in chars.by_ref() {
                    match c {
                        // 下划线⇒加到「前缀」中
                        '_' => prefix.push('_'),
                        // 其它⇒追加到「名称」中，并停止判断下划线
                        _ => {
                            name.push(c);
                            break;
                        }
                    }
                }
                // 后续全部作为「词项名」追加
                for c in chars {
                    name.push(c)
                }
            }
            _ => unreachable!("不可达规则 @ 原子词项 {:?} {pair}", pair.as_rule()),
        }
    }
    Ok(Term::Atom { prefix, name })
}

/// 折叠[`pest`]一元复合词项
fn fold_pest_compound_unary(pair: Pair<Rule>) -> Result<Term> {
    // ! 一元复合结构保证：符号+词项
    let mut pairs = pair.into_inner();
    // 🚩顺序折叠
    let connecter = pairs.next().unwrap().as_str().to_string();
    let terms = vec![fold_pest_term(pairs.next().unwrap())?];
    // 创建
    Ok(Term::Compound { connecter, terms })
}

/// 折叠[`pest`]二元复合词项
/// * 🚩【2024-05-15 01:12:46】此处仅利用「陈述仅有两个子项」存放数据
///   * 📌实际上在NARust中仍然被当作「复合词项」使
fn fold_pest_compound_binary(pair: Pair<Rule>) -> Result<Term> {
    // ! 二元复合结构保证：左+符号+右
    let mut pairs = pair.into_inner();
    // 🚩顺序折叠
    let subject = fold_pest_term(pairs.next().unwrap())?;
    let copula = pairs.next().unwrap().as_str();
    let predicate = fold_pest_term(pairs.next().unwrap())?;
    // 创建
    Ok(Term::new_statement(copula, subject, predicate))
}

/// 折叠[`pest`]多元复合词项
/// * 🚩【2024-05-15 01:03:25】对「集合词项」不做特别兼容：仅为「符号特殊的复合词项」
///   * 📄如：`{A, B, C}` ⇒ `{}(A B C)` ⇒ `{}` + `A` + `B` + `C`
///     * 📌其中`{}`作为复合词项的「连接词」或「连接符」，直接对接NARust的内部表示
fn fold_pest_compound_multi(pair: Pair<Rule>) -> Result<Term> {
    // ! 多元复合结构保证：符号+词项组
    let mut pairs = pair.into_inner();
    // 🚩顺序折叠
    let connecter = pairs.next().unwrap().as_str().to_string();
    let terms = list![
        (fold_pest_term(pair)?)
        for pair in (pairs)
    ];
    // 创建
    Ok(Term::Compound { connecter, terms })
}

/// 词法重整
/// * 🎯将其中的「词法Narsese词项」整理成【可被[`super::_conversion`]解析】的形式
///   * 📌转换后可直接用于「词法折叠」
/// * ❓【2024-05-15 02:22:30】或许可能要绕过「词法Narsese」这层，直接「词项→词项」解析
///   * 💫难点：后续对一个整体内的「语句」「任务」该如何准备？
fn reform_term(original: Term) -> Term {
    // * 🚩解构分类
    use Term::*;
    match original {
        // * 🚩原子词项：直接返回
        Atom { prefix, name } => Atom { prefix, name },
        // * 🚩【2024-05-15 02:21:38】目前并不会构造到这个
        Set { .. } => unreachable!("集合词项不应出现在此处！"),
        // * 🚩从「二元复合词项」中来的「陈述」 ⇒ 陈述 | 复合词项
        Statement {
            copula,
            subject,
            predicate,
        } => match super::Term::is_statement_identifier(&copula) {
            // * 🚩标识符∈陈述 ⇒ 作为「陈述」被解析（递归重整并返回）
            true => Statement {
                copula,
                subject: Box::new(reform_term(*subject)),
                predicate: Box::new(reform_term(*predicate)),
            },
            // * 🚩其它情况：转换为「二元复合词项」 | ⚠️注意：二元的「集合」也需要考虑
            false => match copula.as_str() {
                // * 🚩外延集の标识符 ⇒ 作为「外延集」转换为「集合词项」
                SET_EXT_OPERATOR => Set {
                    left_bracket: SET_EXT_OPENER.into(),
                    terms: vec![reform_term(*subject), reform_term(*predicate)],
                    right_bracket: SET_EXT_CLOSER.into(),
                },
                // * 🚩内涵集の标识符 ⇒ 作为「内涵集」转换为「集合词项」
                SET_INT_OPERATOR => Set {
                    left_bracket: SET_INT_OPENER.into(),
                    terms: vec![reform_term(*subject), reform_term(*predicate)],
                    right_bracket: SET_INT_CLOSER.into(),
                },
                // * 🚩其它⇒一律视作「常规复合词项」递归重整并返回
                _ => Compound {
                    connecter: copula,
                    terms: vec![reform_term(*subject), reform_term(*predicate)],
                },
            },
        },
        // * 🚩从「一元复合词项/多元复合词项」来的「复合词项」 ⇒ 复合词项 | 集合词项
        Compound { connecter, terms } => match connecter.as_str() {
            // * 🚩外延集の标识符 ⇒ 作为「外延集」转换为「集合词项」
            SET_EXT_OPERATOR => Set {
                left_bracket: SET_EXT_OPENER.into(),
                terms: terms.into_iter().map(reform_term).collect(),
                right_bracket: SET_EXT_CLOSER.into(),
            },
            // * 🚩内涵集の标识符 ⇒ 作为「内涵集」转换为「集合词项」
            SET_INT_OPERATOR => Set {
                left_bracket: SET_INT_OPENER.into(),
                terms: terms.into_iter().map(reform_term).collect(),
                right_bracket: SET_INT_CLOSER.into(),
            },
            // * 🚩其它⇒一律视作「复合词项」递归重整并返回
            _ => Compound {
                connecter,
                terms: terms.into_iter().map(reform_term).collect(),
            },
        },
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{global::tests::AResult, ToDisplayAndBrief};
    use narsese::{
        api::NarseseValue, conversion::string::impl_lexical::format_instances::FORMAT_ASCII,
    };

    /// 测试/方言解析器 🚧
    #[test]
    fn test_dialect_parser() -> AResult {
        let narseses = r#"
        word
        $i_var
        #d_var
        ?q_var
        137
        go-to

        {}(SELF)
        [](good)
        &(a b)
        |(a b)
        (a - b)
        (a ~ b)
        *({}(SELF) [](good))

        \(a _ b)
        /(D _ D)
        &&(a b ||(a b c))
        (-- neg)

        (swam --> bird)
        ('文字，文字'-->'/* ~标点 --> 符号! */')
        (a`<=>`b)
        ((a ==> b)<->( a <=> b ))
        ((a {-- b) {-] (a --] b))

        (a`一段文字，但实际上是陈述系词`b)! 
        $$ &/(('@v@'-->b) (b`继承`c) *(b (^c <=> d)) +1 (-- n)). :|: %1.0; 0.9%
        "#
        // 初步数据处理
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty());

        // 开始测试解析
        for narsese in narseses {
            let parsed = parse_lexical(narsese).expect("pest解析失败！");
            // * 🚩词项⇒进一步解析 & 展示
            if let NarseseValue::Term(term) = parsed {
                let parsed_term = crate::language::Term::from_lexical(term)?;
                // 对齐并展示
                println!("    {narsese:?}\n => {:?}", parsed_term.to_display_long());
            }
            // * 🚩其它⇒直接打印字符串
            else {
                let parsed_str = FORMAT_ASCII.format_narsese(&parsed);
                // 对齐并展示
                println!("    {narsese:?}\n => {:?}", parsed_str);
            }
        }

        Ok(println!("测试完毕！"))
    }
}
