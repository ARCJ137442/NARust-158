use super::{GoalV1, JudgementV1, QuestionV1};
use crate::{
    __impl_to_display_and_display,
    entity::{
        sentence::{PunctuatedSentenceRef, Punctuation, Sentence, SentenceInner},
        Stamp, TruthValue,
    },
    global::{ClockTime, Float, OccurrenceTime},
    inference::Evidential,
    language::Term,
};
use anyhow::Result;
use nar_dev_utils::enum_union;
use narsese::lexical::Sentence as LexicalSentence;
use serde::{Deserialize, Serialize};

// 使用「枚举联合」快捷实现「判断/问题」的「语句」类型
// 等效于以下代码：
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum SentenceV1 {
//     JudgementV1(JudgementV1),
//     QuestionV1(QuestionV1),
// }
enum_union! {
    /// 作为【可能是判断，也可能是问题】的统一「语句」类型
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub SentenceV1 = JudgementV1 | QuestionV1 | GoalV1;
}

impl SentenceV1 {
    /// 🆕通过词项、标点、真值、时间戳、可修正 构造 语句
    /// * 🚩根据「标点」分发到各具体类型
    /// * 💭应该挑出到「语句」之外，但暂且放置于此
    /// * 🎯自「导出结论」和「输入构造」被使用
    pub fn with_punctuation(
        new_content: Term,
        punctuation: Punctuation,
        new_stamp: Stamp,
        truth_revisable: Option<(TruthValue, bool)>,
        occurrence_time: OccurrenceTime,
        occurrence_time_offset: Float,
    ) -> Result<Self> {
        use Punctuation::*;
        let sentence = match (punctuation, truth_revisable) {
            // 判断
            (Judgement, Some((new_truth, revisable))) => JudgementV1::new(
                new_content,
                new_truth,
                new_stamp,
                revisable,
                occurrence_time,
                occurrence_time_offset,
            )
            .into(),
            // 问题
            (Question, _) => QuestionV1::new(
                new_content,
                new_stamp,
                occurrence_time,
                occurrence_time_offset,
            )
            .into(),
            // 目标
            (Goal, Some((new_truth, revisable))) => GoalV1::new(
                new_content,
                new_truth,
                new_stamp,
                revisable,
                occurrence_time,
                occurrence_time_offset,
            )
            .into(),
            // 无效
            (Judgement | Goal, None) => Err(anyhow::anyhow!(
                "无效的语句：{punctuation:?}, {truth_revisable:?}"
            ))?,
        };
        Ok(sentence)
    }

    /// 不论是何种类型（判断/问题），获取其中的「内部语句」
    fn inner(&self) -> &SentenceInner {
        match self {
            SentenceV1::JudgementV1(JudgementV1 { inner, .. })
            | SentenceV1::QuestionV1(QuestionV1 { inner, .. })
            | SentenceV1::GoalV1(GoalV1 { inner, .. }) => inner,
        }
    }

    /// 不论是何种类型（判断/问题），获取其中的「内部语句」
    fn inner_mut(&mut self) -> &mut SentenceInner {
        match self {
            SentenceV1::JudgementV1(JudgementV1 { inner, .. })
            | SentenceV1::QuestionV1(QuestionV1 { inner, .. })
            | SentenceV1::GoalV1(GoalV1 { inner, .. }) => inner,
        }
    }
}

impl Evidential for SentenceV1 {
    fn evidential_base(&self) -> &[ClockTime] {
        self.inner().stamp().evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.inner().stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner().stamp().stamp_to_lexical()
    }
}

/// 将「语句」用内部的变种类型代理
macro_rules! as_variant {
    {$this:expr, $name:ident => $($code:tt)*} => {
        match $this {
            SentenceV1::JudgementV1($name) => $($code)*,
            SentenceV1::QuestionV1($name) => $($code)*,
            SentenceV1::GoalV1($name) => $($code)*,
        }
    };
}

impl Sentence for SentenceV1 {
    fn sentence_clone<'s, 'sentence: 's>(&'s self) -> impl Sentence + 'sentence {
        self.clone()
    }

    fn content(&self) -> &Term {
        self.inner().content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner_mut().content_mut()
    }

    type Judgement = JudgementV1;
    type Question = QuestionV1;
    type Goal = GoalV1;

    /// ℹ️只需这一个方法，即可提供所有与「细分类型/标点」有关的信息
    fn as_punctuated_ref(
        &self,
    ) -> PunctuatedSentenceRef<Self::Judgement, Self::Question, Self::Goal> {
        use PunctuatedSentenceRef::*;
        use SentenceV1::*;
        match self {
            JudgementV1(j) => Judgement(j),
            QuestionV1(q) => Question(q),
            GoalV1(q) => Goal(q),
        }
    }

    fn to_key(&self) -> String {
        // 直接朝内部分派
        as_variant! {
            self, s => s.to_key()
        }
    }

    fn occurrence_time(&self) -> OccurrenceTime {
        self.inner().occurrence_time
    }

    fn occurrence_time_offset(&self) -> Float {
        self.inner().occurrence_time_offset
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
// * new_sentence_from_punctuation
// * get_inner
// * get_inner_mut
// * evidential_base
// * creation_time
// * stamp_to_lexical
// * sentence_clone
// * content
// * content_mut
// * as_punctuated_ref
// * to_key
// * sentence_to_display
// * sentence_to_lexical
