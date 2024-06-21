use super::{Judgement, QuestionV1, Sentence, SentenceInner, Stamp, TruthValue};
use crate::{
    __impl_to_display_and_display,
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
    type Judgement = Self;
    type Question = QuestionV1;
    fn content(&self) -> &Term {
        self.inner.content()
    }

    fn content_mut(&mut self) -> &mut Term {
        self.inner.content_mut()
    }

    fn punctuation(&self) -> super::Punctuation {
        super::Punctuation::Judgement
    }

    fn is_judgement(&self) -> bool {
        true
    }

    fn as_judgement(&self) -> Option<&Self::Judgement> {
        Some(self)
    }

    fn is_question(&self) -> bool {
        false
    }

    fn as_question(&self) -> Option<&Self::Question> {
        None
    }

    fn to_key(&self) -> String {
        todo!()
    }

    fn sentence_to_display(&self) -> String {
        todo!()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        todo!()
    }
}

impl Truth for JudgementV1 {
    #[inline(always)]
    fn frequency(&self) -> super::ShortFloat {
        self.truth.frequency()
    }

    #[inline(always)]
    fn frequency_mut(&mut self) -> &mut super::ShortFloat {
        self.truth.frequency_mut()
    }

    #[inline(always)]
    fn confidence(&self) -> super::ShortFloat {
        self.truth.confidence()
    }

    #[inline(always)]
    fn confidence_mut(&mut self) -> &mut super::ShortFloat {
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
