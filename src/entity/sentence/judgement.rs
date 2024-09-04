use super::Sentence;
use crate::inference::Truth;
use nar_dev_utils::join;
use narsese::lexical::Sentence as LexicalSentence;

/// 统一的「判断句」特征
/// * 🎯通用地表示「语句+真值」的概念
/// * 📌在[「语句」](Sentence)的基础上具有「可修正」等功能
pub trait Judgement: Sentence + Truth {
    /// 📄改版OpenNARS `static revisable`
    fn revisable_to(&self, other: &Self) -> bool {
        let content_eq = self.content() == other.content();
        let other_revisable = other.revisable();
        content_eq && other_revisable
    }

    /// 模拟`Sentence.revisable`、`Sentence.getRevisable`
    /// * 📝OpenNARS只在「解析任务」时会设置值
    ///   * 🎯使用目的：「包含因变量的合取」不可被修正
    ///   * 🚩【2024-05-19 13:01:57】故无需让其可变，构造后只读即可
    /// * 🚩【2024-05-24 12:05:54】现在将「是否可修正」放进「判断」标点中
    ///   * 📝根据OpenNARS逻辑，只有「判断」才有「是否可被修正」属性
    ///   * ✅现在无需再依靠具体结构来实现了
    /// * 🚩【2024-06-21 14:54:59】现在成为抽象方法
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

    fn is_belief_equivalent(&self, other: &impl Judgement) -> bool {
        self.truth_eq(other) && self.evidential_eq(other)
    }

    // ! ❌不能在此自动实现`toKey` `sentenceToString`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法
    /// 作为一个[`Sentence::to_key`]的默认【非覆盖性】实现
    fn judgement_to_key(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display_brief()
        }
    }

    /// 作为一个[`Sentence::sentence_to_display`]的默认【非覆盖性】实现
    fn judgement_to_display(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string() + " "
            => self.truth_to_display()
            => self.stamp_to_display()
        }
    }

    /// 作为一个[`Sentence::to_lexical`]的默认【非覆盖性】实现
    fn judgement_to_lexical(&self) -> LexicalSentence {
        LexicalSentence {
            term: self.content().into(),
            // 标点：采用字符串形式
            punctuation: self.punctuation().to_char().into(),
            stamp: self.stamp_to_lexical(),
            // 判断句有真值
            truth: self.truth_to_lexical(),
        }
    }
}
