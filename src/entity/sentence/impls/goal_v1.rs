//! 初代目标句实现

use crate::entity::{
    Goal, JudgementV1, PunctuatedSentenceRef, QuestionV1, Sentence, SentenceInner,
};
use crate::global::{Float, OccurrenceTime};
use crate::{
    __impl_to_display_and_display,
    entity::{ShortFloat, Stamp, TruthValue},
    global::ClockTime,
    inference::{Evidential, Truth},
    language::Term,
};
use narsese::lexical::Sentence as LexicalSentence;
use serde::{Deserialize, Serialize};

/// 🆕目标句 初代实现
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalV1 {
    /// 🆕内部存储的「语句」实现
    pub(crate) inner: SentenceInner,
    /// Whether the sentence can be revised
    revisable: bool,
    /// The desire value of Judgment as truth
    truth: TruthValue,
}

impl GoalV1 {
    pub fn new(
        content: Term,
        truth: impl Into<TruthValue>,
        stamp: impl Into<Stamp>,
        revisable: bool,
        occurrence_time: OccurrenceTime,
        occurrence_time_offset: Float,
    ) -> Self {
        Self {
            inner: SentenceInner::new(
                content,
                stamp.into(),
                occurrence_time,
                occurrence_time_offset,
            ),
            revisable,
            truth: truth.into(),
        }
    }
}

impl Evidential for GoalV1 {
    fn evidential_base(&self) -> &[ClockTime] {
        self.inner.stamp().evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.inner.stamp().creation_time()
    }
}

impl Sentence for GoalV1 {
    fn sentence_clone<'s, 'sentence: 's>(&'s self) -> impl Sentence + 'sentence {
        self.clone()
    }

    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    type Judgement = JudgementV1;
    type Question = QuestionV1;
    type Goal = Self;

    #[inline(always)]
    fn as_punctuated_ref(
        &self,
    ) -> PunctuatedSentenceRef<Self::Judgement, Self::Question, Self::Goal> {
        PunctuatedSentenceRef::Goal(self)
    }

    fn to_key(&self) -> String {
        self.goal_to_key()
    }

    fn occurrence_time(&self) -> OccurrenceTime {
        self.inner.occurrence_time
    }

    fn occurrence_time_offset(&self) -> Float {
        self.inner.occurrence_time_offset
    }

    fn sentence_to_display(&self) -> String {
        self.goal_to_display()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner.stamp_to_lexical()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.goal_to_lexical()
    }
}

impl Truth for GoalV1 {
    #[inline(always)]
    fn frequency(&self) -> ShortFloat {
        self.truth.frequency()
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut ShortFloat {
        self.truth.frequency_mut()
    }

    #[inline(always)]
    fn confidence(&self) -> ShortFloat {
        self.truth.confidence()
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut ShortFloat {
        self.truth.confidence_mut()
    }

    #[inline(always)]
    fn is_analytic(&self) -> bool {
        self.truth.is_analytic()
    }

    #[inline(always)]
    fn set_analytic(&mut self) {
        self.truth.set_analytic()
    }
}

impl Goal for GoalV1 {
    fn revisable(&self) -> bool {
        self.revisable
    }
}

__impl_to_display_and_display! {
    @(goal_to_display;;)
    GoalV1 as Goal
}

// TODO: 单元测试
// * new
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
// * frequency
// * frequency_mut
// * confidence
// * confidence_mut
// * is_analytic
// * set_analytic
// * revisable
