//! 实现 / 构造

use super::structs::*;
use crate::symbols::*;
use anyhow::Result;
use nar_dev_utils::*;

impl Term {
    /// 构造函数
    /// * ⚠️有限性：仅限在「内部」使用，不希望外部以此构造出「不符范围」的词项
    /// * 📌【2024-09-07 13:07:41】进一步限定可见性：只在**当前模块**中使用
    fn new(identifier: impl Into<String>, components: TermComponents) -> Self {
        Self {
            identifier: identifier.into(),
            components,
        }
    }

    // 原子词项 //
    // * ℹ️此处一系列构造方法对标OpenNARS中各「词项」的构造函数
    // * ⚠️在MakeTerm中另有一套方法（参见term_making.rs）

    /// NAL-1 / 词语
    pub(super) fn new_word(name: impl Into<String>) -> Self {
        Self::new(WORD, TermComponents::Word(name.into()))
    }

    /// NAL-4 / 占位符
    /// * 📌【2024-04-21 00:36:27】需要一个「占位符」词项，以便和「词法Narsese」打交道
    /// * 🚩仅使用「占位符标识符+空组分」表示
    /// * 🎯仅在解析时临时出现
    /// * ⚠️【2024-04-25 09:45:51】不允许外部直接创建
    pub(super) fn new_placeholder() -> Self {
        Self::new(PLACEHOLDER, TermComponents::Empty)
    }

    /// NAL-6 / 变量（内部统一代码）
    /// * ℹ️外部统一使用[`Self::from_var_similar`]
    fn new_var(identifier: impl Into<String>, id: impl Into<usize>) -> Self {
        Self::new(identifier.into(), TermComponents::Variable(id.into()))
    }

    /// NAL-6 / 独立变量
    pub(super) fn new_var_i(id: impl Into<usize>) -> Self {
        Self::new_var(VAR_INDEPENDENT, id.into())
    }

    /// NAL-6 / 非独变量
    pub(super) fn new_var_d(id: impl Into<usize>) -> Self {
        Self::new_var(VAR_DEPENDENT, id.into())
    }

    /// NAL-6 / 查询变量
    pub(super) fn new_var_q(id: impl Into<usize>) -> Self {
        Self::new_var(VAR_QUERY, id.into())
    }

    /// 从「变量类型」与「id」构造一个变量
    /// * 🎯在「变量替换」中创建新变量
    ///   * 📌【2024-09-07 16:17:57】因为外部「变量处理」要用到，此处暂且放开
    /// * ⚠️【2024-04-25 23:08:20】内部使用：会导致产生无效类型（改变了组分类型）
    pub(in super::super) fn from_var_similar(
        var_type: impl Into<String>,
        new_id: impl Into<usize>,
    ) -> Self {
        Self::new_var(var_type, new_id)
    }

    // 复合词项 //

    /// NAL-3 / 外延集
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_set_ext(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(SET_EXT_OPERATOR, TermComponents::new_multi(terms.into()))
    }

    /// NAL-3 / 内涵集
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_set_int(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(SET_INT_OPERATOR, TermComponents::new_multi(terms.into()))
    }

    /// NAL-3 / 外延交
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_intersection_ext(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            INTERSECTION_EXT_OPERATOR,
            TermComponents::new_multi(terms.into()),
        )
    }

    /// NAL-3 / 内涵交
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_intersection_int(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            INTERSECTION_INT_OPERATOR,
            TermComponents::new_multi(terms.into()),
        )
    }

    /// NAL-3 / 外延差
    pub(super) fn new_diff_ext(term1: Term, term2: Term) -> Self {
        Self::new(
            DIFFERENCE_EXT_OPERATOR,
            TermComponents::new_binary(term1, term2),
        )
    }

    /// NAL-3 / 内涵差
    pub(super) fn new_diff_int(term1: Term, term2: Term) -> Self {
        Self::new(
            DIFFERENCE_INT_OPERATOR,
            TermComponents::new_binary(term1, term2),
        )
    }

    /// NAL-4 / 乘积
    pub(super) fn new_product(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(PRODUCT_OPERATOR, TermComponents::new_multi(terms.into()))
    }

    /// NAL-4 / 外延像
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    pub(super) fn new_image_ext(terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_EXT_OPERATOR,
            Self::_process_image_terms(terms)?,
        ))
    }

    /// NAL-4 / 内涵像
    /// * 📝占位符索引≠关系词项索引（in OpenNARS）
    ///   * ⚠️占位符索引=0 ⇒ 不被允许
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    pub(super) fn new_image_int(terms: impl Into<Vec<Term>>) -> Result<Self> {
        Ok(Self::new(
            IMAGE_INT_OPERATOR,
            Self::_process_image_terms(terms)?,
        ))
    }

    /// 代码复用之工具函数：处理像占位符和词项列表
    /// * 🚩将词项列表转换为`Vec<Term>`
    /// * 🚩检查占位符索引范围
    /// * 🚩返回构造好的「词项组分」
    /// * ⚠️会返回错误
    ///
    /// ! ⚠️【2024-06-16 16:50:23】现在传入的「词项列表」将附带「像占位符」词项
    #[inline(always)]
    fn _process_image_terms(terms: impl Into<Vec<Term>>) -> Result<TermComponents> {
        // 转换词项列表
        let terms = terms.into();
        // 检索像占位符位置
        let i_placeholder = terms.iter().position(Term::is_placeholder);
        // 检查占位符索引范围
        match i_placeholder {
            Some(i_placeholder) => {
                // * ✅`terms.iter().position`保证：占位符索引不会超出范围
                if i_placeholder == 0 {
                    return Err(anyhow::anyhow!("占位符不能压在「关系词项」的位置上"));
                }
            }
            None => return Err(anyhow::anyhow!("未在像的元素中找到占位符")),
        }
        // 构造 & 返回
        // * 🚩【2024-06-12 22:48:33】现在不再附带额外字段，统一使用一个枚举变种
        Ok(TermComponents::new_multi(terms))
    }

    /// NAL-5 / 合取
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_conjunction(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            CONJUNCTION_OPERATOR,
            TermComponents::new_multi(terms.into()),
        )
    }

    /// NAL-5 / 析取
    /// * 🚩【2024-04-21 13:39:28】使用统一的「无序不重复集合」构造组分
    pub(super) fn new_disjunction(terms: impl Into<Vec<Term>>) -> Self {
        Self::new(
            DISJUNCTION_OPERATOR,
            TermComponents::new_multi(terms.into()),
        )
    }

    /// NAL-5 / 否定
    pub(super) fn new_negation(term: Term) -> Self {
        Self::new(NEGATION_OPERATOR, TermComponents::new_unary(term))
    }

    // 陈述 //

    /// NAL-1 / 继承
    pub(super) fn new_inheritance(subject: Term, predicate: Term) -> Self {
        Self::new(
            INHERITANCE_RELATION,
            TermComponents::new_binary(subject, predicate),
        )
    }

    /// NAL-3 / 相似
    pub(super) fn new_similarity(subject: Term, predicate: Term) -> Self {
        Self::new(
            SIMILARITY_RELATION,
            TermComponents::new_binary_unordered(subject, predicate),
        )
    }

    /// NAL-5 / 蕴含
    pub(super) fn new_implication(subject: Term, predicate: Term) -> Self {
        Self::new(
            IMPLICATION_RELATION,
            TermComponents::new_binary(subject, predicate),
        )
    }

    /// NAL-5 / 等价
    pub(super) fn new_equivalence(subject: Term, predicate: Term) -> Self {
        Self::new(
            EQUIVALENCE_RELATION,
            TermComponents::new_binary_unordered(subject, predicate),
        )
    }
}

impl TermComponents {
    /// 一元组分
    /// * 🚩【2024-06-12 22:43:34】现在封装「内部枚举变种」接口
    pub(super) fn new_unary(term: Term) -> Self {
        Self::Compound(Box::new([term]))
    }

    /// 二元有序组分
    /// * 🚩【2024-06-12 22:43:34】现在封装「内部枚举变种」接口
    pub(super) fn new_binary(term1: Term, term2: Term) -> Self {
        Self::Compound(Box::new([term1, term2]))
    }

    /// 二元无序组分
    /// * 🎯用于【双元素对称性】复合词项
    /// * ⚠️无法去重：元素数量固定为`2`
    /// * 📄相似、等价
    /// * 🚩使用「临时数组切片」实现（较为简洁）
    pub(super) fn new_binary_unordered(term1: Term, term2: Term) -> Self {
        pipe! {
            // 排序
            manipulate!(
                [term1, term2]
                => .sort()
            )
            // 构造
            => Box::new
            => Self::Compound
        }
    }

    /// 多元组分
    /// * 📌兼容「有序可重复」「无序不重复」两种
    /// * 🚩【2024-09-07 17:27:00】现在将「无序不重复组分」外包到[`super::making`]模块中
    ///   * 📌在外部保证「有序性/无序性/可重复性/不重复性」
    pub(super) fn new_multi(terms: Vec<Term>) -> Self {
        pipe! {
            terms
            // 转换
            => .into_boxed_slice()
            // 构造
            => Self::Compound
        }
    }
}

/// [「词项」](Term)的快捷构造宏
#[macro_export]
macro_rules! term {
    // 单个词项（字符串）
    ($s:literal) => {
        $s.parse::<$crate::language::Term>()
    };
    // 单个词项，但unwrap
    (unwrap $s:expr) => {
        $s.parse::<$crate::language::Term>().unwrap()
    };
    // 单个词项，无需引号
    ($($t:tt)*) => {
        stringify!($($t)*).parse::<$crate::language::Term>()
    };
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ok, test_term as t, util::AResult};
    use nar_dev_utils::fail_tests;
    // ! ❌使用`test_term as t`避免`term`重名：即便不导入，也会ambiguous

    /// 测试/词项
    #[test]
    fn test_term() -> AResult {
        // 测试一个词项
        fn detect(term: &Term) {
            use TermComponents::*;
            match term.id_comp() {
                (WORD, Word(name)) => {
                    println!("word with {name:?}");
                }
                (IMAGE_EXT_OPERATOR, Compound(v)) => {
                    let i = v.iter().position(Term::is_placeholder).unwrap();
                    println!("ext_image '/' with {i}");
                    println!("<components>");
                    for term in v.iter() {
                        detect(term);
                    }
                    println!("</components>");
                }
                _ => println!("term {:?}: {}", term.identifier, term.format_name()),
            }
        }
        // 直接从内部构造函数中构造一个词项
        let im_ext = Term::new(
            IMAGE_EXT_OPERATOR,
            TermComponents::new_multi(vec![Term::new_word("word"), Term::new_placeholder()]),
        );
        detect(&im_ext);
        // 从「词法Narsese」中解析词项
        detect(&t!("<A --> B>"));
        detect(&t!("(--, [C, B, A, 0, 1, 2])"));
        detect(&t!(
            "{<B <-> A>, <D <=> C>, (&&, <A --> B>, <B --> C>), $i, #d, ?q}"
        ));
        detect(&t!("(/, _, A, B)"));
        detect(&t!("(/, A, _, B)"));
        detect(&t!("(/, A, B, _)"));
        detect(&t!(r"(\, _, A, B)"));
        detect(&t!(r"(\, A, _, B)"));
        detect(&t!(r"(\, A, B, _)"));
        // 返回成功
        ok!()
    }

    // 失败测试
    fail_tests! {
        组分数不对_二元_外延差1 t!(unwrap "(-, A)");
        组分数不对_二元_外延差3 t!(unwrap "(-, A, B, C)");
        组分数不对_一元_否定 t!(unwrap "(--, A, B)");
        空集_外延集 t!(unwrap "{}");
        空集_内涵集 t!(unwrap "[]");
        空集_外延像 t!(unwrap r"(/, _)");
        空集_内涵像 t!(unwrap r"(\, _)");
    }
}
