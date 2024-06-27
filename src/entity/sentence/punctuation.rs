//! 存放特定的「标点」类型

use crate::io::symbols::*;
use anyhow::Result;
use nar_dev_utils::unwrap_or_return;
use narsese::lexical::Punctuation as LexicalPunctuation;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// NARust特制的「标点」类型
/// * 📌相比旧版的`SentenceType`，此处仅提供简单枚举，不附带字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punctuation {
    Judgement,
    Question,
}

impl Punctuation {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            JUDGMENT_MARK => Some(Self::Judgement),
            QUESTION_MARK => Some(Self::Question),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        use Punctuation::*;
        match self {
            Judgement => JUDGMENT_MARK,
            Question => QUESTION_MARK,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Punctuation::*;
        match self {
            Judgement => write!(f, "{JUDGMENT_MARK}"),
            Question => write!(f, "{QUESTION_MARK}"),
        }
    }
}
