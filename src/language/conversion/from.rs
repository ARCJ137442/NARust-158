//! 词项→其它类型

use super::{super::base::*, lexical_fold};
use anyhow::Result;
use narsese::{conversion::inter_type::lexical_fold::TryFoldInto, lexical::Term as TermLexical};
use std::str::FromStr;

impl Term {
    /// 尝试从「词法Narsese」转换
    /// * 💭【2024-04-21 14:44:15】目前此中方法「相较保守」
    /// * 📌与词法Narsese基本对应（ASCII）
    /// * ✅基本保证「解析结果均保证『合法』」
    /// * 🚩【2024-06-13 18:39:33】现在是「词法折叠」使用本处实现
    /// * ⚠️在「词法折叠」的过程中，即开始「变量匿名化」
    ///   * 📌【2024-07-02 00:40:39】需要保证「格式化」的是个「整体」：变量只在「整体」范围内有意义
    /// * 🚩【2024-09-06 17:32:12】在「词法折叠」的过程中，即开始使用`make`系列方法
    ///   * 🎯应对类似「`(&&, A, A)` => `(&&, A)`」的「不完整简化」现象
    #[inline]
    pub fn from_lexical(lexical: TermLexical) -> Result<Self> {
        lexical_fold::lexical_fold(lexical)
    }

    /// 尝试从「方言」转换
    /// * 🎯支持「方言解析」
    /// * 📌【2024-05-15 02:33:13】目前仍只有「从字符串到词项」这一种形式
    /// * 🆕附加功能，与核心「数据算法」「推理控制」无关
    #[inline]
    #[cfg(feature = "dialect_parser")]
    pub fn from_dialect(input: &str) -> Result<Self> {
        use super::super::dialect::parse_term;
        parse_term(input)
    }
}

/// 词法折叠
impl TryFoldInto<'_, Term, anyhow::Error> for TermLexical {
    /// 类型占位符
    type Folder = ();

    fn try_fold_into(self, _: &'_ Self::Folder) -> Result<Term> {
        Term::from_lexical(self)
    }
}

/// 基于「词法折叠」实现[`TryFrom`]
impl TryFrom<TermLexical> for Term {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: TermLexical) -> Result<Self, Self::Error> {
        value.try_fold_into(&())
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
