//! 存放特定的「标点」类型

use super::{Judgement, Question};
use crate::symbols::*;
use anyhow::Result;
use nar_dev_utils::unwrap_or_return;
use narsese::lexical::Punctuation as LexicalPunctuation;
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// NARust特制的「标点」类型
/// * 📌相比旧版的`SentenceType`，此处仅提供简单枚举，不附带字段
///   * 🎯为后续一些「无需使用内部数据」的场合提供便利
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punctuation {
    /// 判断
    Judgement,
    /// 疑问
    Question,
    /// 目标
    Goal,
}

impl Punctuation {
    /// 从字符转换
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            JUDGMENT_MARK => Some(Self::Judgement),
            QUESTION_MARK => Some(Self::Question),
            GOAL_MARK => Some(Self::Goal),
            _ => None,
        }
    }

    /// 转换到字符
    pub const fn to_char(&self) -> char {
        use Punctuation::*;
        match self {
            Judgement => JUDGMENT_MARK,
            Question => QUESTION_MARK,
            Goal => GOAL_MARK,
        }
    }

    /// 从「词法Narsese」中转换
    /// * 🚩以末尾字符为标准
    pub fn from_lexical(lexical: LexicalPunctuation) -> Result<Self> {
        // * 🚩获取字符
        let last_char = unwrap_or_return! {
            ?lexical.chars().next_back()
            => Err(anyhow::anyhow!("标点字符串为空"))
        };
        // * 🚩解析字符
        Self::from_char(last_char)
            .ok_or_else(|| anyhow::anyhow!("无效的标点字符: {last_char} @ {lexical}"))
    }
}

impl Display for Punctuation {
    /// 直接写入自身对应字符
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_char(self.to_char())
    }
}

/// 带标点的「特定类型语句」引用
/// * 📌不可变引用
/// * 🎯在「标点」的基础上，附带更有用的匹配信息
///   * 📄减少非必要（且不易稳定）的`unwrap`，用类型系统规范使用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PunctuatedSentenceRef<'r, J, Q>
where
    J: Judgement,
    Q: Question,
{
    /// 判断
    Judgement(&'r J),
    /// 疑问
    Question(&'r Q),
}

impl<'r, J, Q> PunctuatedSentenceRef<'r, J, Q>
where
    J: Judgement,
    Q: Question,
{
    /// 转换到【纯粹作为标签存在】的[`Punctuation`]
    pub const fn to_punctuation(&self) -> Punctuation {
        use PunctuatedSentenceRef::*;
        match self {
            Judgement(..) => Punctuation::Judgement,
            Question(..) => Punctuation::Question,
        }
    }

    /// 先转换到「纯标点」再转换到字符
    #[inline(always)]
    pub const fn to_char(&self) -> char {
        self.to_punctuation().to_char()
    }
}

/// 派生性实现[`From`]
impl<'r, J, Q> From<PunctuatedSentenceRef<'r, J, Q>> for Punctuation
where
    J: Judgement,
    Q: Question,
{
    fn from(value: PunctuatedSentenceRef<'r, J, Q>) -> Self {
        value.to_punctuation()
    }
}

/// 派生性实现[`Display`]
impl<'r, J, Q> Display for PunctuatedSentenceRef<'r, J, Q>
where
    J: Judgement,
    Q: Question,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use PunctuatedSentenceRef::*;
        let sentence = match self {
            Judgement(sentence) => sentence.sentence_to_display(),
            Question(sentence) => sentence.sentence_to_display(),
        };
        write!(f, "ref{:?} @ ({sentence})", self.to_char())
    }
}
