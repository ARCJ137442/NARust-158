use super::Sentence;
use crate::inference::Truth;
use nar_dev_utils::join;
use narsese::lexical::Sentence as LexicalSentence;

/// 统一的「目标句」特征
/// * 🎯通用地表示「语句+真值」的概念
/// * 📌在[「语句」](Sentence)的基础上具有「可修正」等功能
pub trait Goal: Sentence + Truth {
    /// 📄改版OpenNARS `static revisable`
    fn revisable_to(&self, other: &Self) -> bool {
        let content_eq = self.content() == other.content();
        let other_revisable = other.revisable();
        content_eq && other_revisable
    }

    /// 模拟`Sentence.revisable`、`Sentence.getRevisable`
    /// * 🚩【2024-10-29 20:38:24】目前按照个人直觉，先将「判断」的属性原封不动移植至「目标」中
    ///
    /// # 📄OpenNARS
    ///
    /// ## `revisable`
    ///
    /// Whether the sentence can be revised
    ///
    /// ## `getRevisable`
    ///
    /// 🈚
    fn revisable(&self) -> bool;

    // ! ❌不能在此自动实现`isJudgement` `asJudgement`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法

    fn is_belief_equivalent(&self, other: &impl Goal) -> bool {
        self.truth_eq(other) && self.evidential_eq(other)
    }

    // ! ❌不能在此自动实现`toKey` `sentenceToString`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法
    /// 作为一个[`Sentence::to_key`]的默认【非覆盖性】实现
    fn goal_to_key(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display_brief()
        }
    }

    /// 作为一个[`Sentence::sentence_to_display`]的默认【非覆盖性】实现
    fn goal_to_display(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display()
            => self.stamp_to_display()
        }
    }

    /// 作为一个[`Sentence::to_lexical`]的默认【非覆盖性】实现
    fn goal_to_lexical(&self) -> LexicalSentence {
        LexicalSentence {
            term: self.content().into(),
            // 标点：采用字符串形式
            punctuation: self.punctuation().to_char().into(),
            stamp: self.stamp_to_lexical(),
            // 目标句有真值
            truth: self.truth_to_lexical(),
        }
    }
}
