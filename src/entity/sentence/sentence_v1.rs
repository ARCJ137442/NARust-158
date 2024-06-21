use super::{JudgementV1, Punctuation, QuestionV1, Sentence, SentenceInner};
use crate::{
    __impl_to_display_and_display,
    entity::{Stamp, TruthValue},
    global::ClockTime,
    inference::Evidential,
    language::Term,
};
use anyhow::Result;
use nar_dev_utils::{enum_union, matches_or};
use narsese::lexical::Sentence as LexicalSentence;

// 使用「枚举联合」快捷实现「判断/问题」的「语句」类型
// 等效于以下代码：
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum SentenceV1 {
//     JudgementV1(JudgementV1),
//     QuestionV1(QuestionV1),
// }
enum_union! {
    /// 作为【可能是判断，也可能是问题】的统一「语句」类型
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub SentenceV1 = JudgementV1 | QuestionV1;
}

impl SentenceV1 {
    /// 🆕通过词项、标点、真值、时间戳、可修正 构造 语句
    /// * 🚩根据「标点」分发到各具体类型
    /// * 💭应该挑出到「语句」之外，但暂且放置于此
    /// * 🎯自「导出结论」和「输入构造」被使用
    pub fn new_sentence_from_punctuation(
        new_content: Term,
        punctuation: Punctuation,
        new_stamp: Stamp,
        truth_revisable: Option<(TruthValue, bool)>,
    ) -> Result<Self> {
        use Punctuation::*;
        let sentence = match (punctuation, truth_revisable) {
            (Judgement, Some((new_truth, revisable))) => {
                JudgementV1::new(new_content, new_truth, new_stamp, revisable).into()
            }
            (Question, ..) => QuestionV1::new(new_content, new_stamp).into(),
            _ => Err(anyhow::anyhow!(
                "无效的语句：{punctuation:?}, {truth_revisable:?}"
            ))?,
        };
        Ok(sentence)
    }

    /// 不论是何种类型（判断/问题），获取其中的「内部语句」
    pub fn get_inner(&self) -> &SentenceInner {
        match self {
            SentenceV1::JudgementV1(JudgementV1 { inner, .. })
            | SentenceV1::QuestionV1(QuestionV1 { inner, .. }) => inner,
        }
    }

    /// 不论是何种类型（判断/问题），获取其中的「内部语句」
    pub fn get_inner_mut(&mut self) -> &mut SentenceInner {
        match self {
            SentenceV1::JudgementV1(JudgementV1 { inner, .. })
            | SentenceV1::QuestionV1(QuestionV1 { inner, .. }) => inner,
        }
    }
}

impl Evidential for SentenceV1 {
    fn evidential_base(&self) -> &[ClockTime] {
        self.get_inner().stamp().evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.get_inner().stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.get_inner().stamp().stamp_to_lexical()
    }
}

/// 将「语句」用内部的变种类型代理
macro_rules! as_variant {
    {$this:expr, $name:ident => $($code:tt)*} => {
        match $this {
            SentenceV1::JudgementV1($name) => $($code)*,
            SentenceV1::QuestionV1($name) => $($code)*,
        }
    };
}

impl Sentence for SentenceV1 {
    fn sentence_clone(&self) -> impl Sentence {
        self.clone()
    }

    fn content(&self) -> &Term {
        self.get_inner().content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.get_inner_mut().content_mut()
    }

    fn punctuation(&self) -> Punctuation {
        // 直接朝内部分派
        as_variant! {
            self, s => s.punctuation()
        }
    }

    type Judgement = JudgementV1;

    type Question = QuestionV1;

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        matches_or! {
            ?self,
            Self::JudgementV1(j) => j
        }
    }

    fn as_question(&self) -> Option<&Self::Question> {
        matches_or! {
            ?self,
            Self::QuestionV1(q) => q
        }
    }

    fn to_key(&self) -> String {
        // 直接朝内部分派
        as_variant! {
            self, s => s.to_key()
        }
    }

    fn sentence_to_display(&self) -> String {
        // 直接朝内部分派
        as_variant! {
            self, s => s.sentence_to_display()
        }
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        as_variant! {
            self, s => s.sentence_to_lexical()
        }
    }
}

__impl_to_display_and_display! {
    @(sentence_to_display;;)
    SentenceV1 as Sentence
}

// TODO: 单元测试
