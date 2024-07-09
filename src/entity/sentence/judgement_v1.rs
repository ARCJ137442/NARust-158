//! 初代判断句实现

use crate::entity::{Judgement, PunctuatedSentenceRef, QuestionV1, Sentence, SentenceInner};
use crate::{
    __impl_to_display_and_display,
    entity::{ShortFloat, Stamp, TruthValue},
    global::ClockTime,
    inference::{Evidential, Truth},
    language::Term,
};
use narsese::lexical::Sentence as LexicalSentence;

/// 🆕判断句 初代实现
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JudgementV1 {
    /// 🆕内部存储的「语句」实现
    pub(crate) inner: SentenceInner,
    /// Whether the sentence can be revised
    revisable: bool,
    /// The truth value of Judgment
    truth: TruthValue,
}

impl JudgementV1 {
    pub fn new(
        content: Term,
        truth: impl Into<TruthValue>,
        stamp: impl Into<Stamp>,
        revisable: bool,
    ) -> Self {
        Self {
            inner: SentenceInner::new(content, stamp.into()),
            revisable,
            truth: truth.into(),
        }
    }
}

impl Evidential for JudgementV1 {
    fn evidential_base(&self) -> &[ClockTime] {
        self.inner.stamp().evidential_base()
    }

    fn creation_time(&self) -> ClockTime {
        self.inner.stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner.stamp().stamp_to_lexical()
    }
}

impl Sentence for JudgementV1 {
    fn sentence_clone(&self) -> impl Sentence {
        self.clone()
    }

    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    type Judgement = Self;
    type Question = QuestionV1;

    #[inline(always)]
    fn as_punctuated_ref(&self) -> PunctuatedSentenceRef<Self::Judgement, Self::Question> {
        PunctuatedSentenceRef::Judgement(self)
    }

    fn to_key(&self) -> String {
        self.judgement_to_key()
    }

    fn sentence_to_display(&self) -> String {
        self.judgement_to_display()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.judgement_to_lexical()
    }
}

impl Truth for JudgementV1 {
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

impl Judgement for JudgementV1 {
    fn revisable(&self) -> bool {
        self.revisable
    }
}

__impl_to_display_and_display! {
    @(judgement_to_display;;)
    JudgementV1 as Judgement
}

// TODO: 单元测试
