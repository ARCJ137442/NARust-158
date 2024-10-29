//! 初代疑问句实现

use crate::{
    __impl_to_display_and_display,
    entity::{
        GoalV1, JudgementV1, PunctuatedSentenceRef, Question, Sentence, SentenceInner, Stamp,
    },
    inference::Evidential,
    language::Term,
};
use narsese::lexical::Sentence as LexicalSentence;
use serde::{Deserialize, Serialize};

/// 🆕疑问句 初代实现
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionV1 {
    pub(crate) inner: SentenceInner,
}

impl QuestionV1 {
    pub fn new(content: Term, stamp: Stamp) -> Self {
        Self {
            inner: SentenceInner::new(content, stamp),
        }
    }
}

impl Evidential for QuestionV1 {
    fn evidential_base(&self) -> &[crate::global::ClockTime] {
        self.inner.stamp().evidential_base()
    }

    fn creation_time(&self) -> crate::global::ClockTime {
        self.inner.stamp().creation_time()
    }

    fn stamp_to_lexical(&self) -> narsese::lexical::Stamp {
        self.inner.stamp().stamp_to_lexical()
    }
}

impl Sentence for QuestionV1 {
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
    type Question = Self;
    type Goal = GoalV1;

    #[inline(always)]
    fn as_punctuated_ref(
        &self,
    ) -> PunctuatedSentenceRef<Self::Judgement, Self::Question, Self::Goal> {
        PunctuatedSentenceRef::Question(self)
    }

    fn to_key(&self) -> String {
        self.question_to_key()
    }

    fn sentence_to_display(&self) -> String {
        self.question_to_display()
    }

    fn sentence_to_lexical(&self) -> LexicalSentence {
        self.question_to_lexical()
    }
}

impl Question for QuestionV1 {}

__impl_to_display_and_display! {
    @(question_to_display;;)
    QuestionV1 as Question
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
