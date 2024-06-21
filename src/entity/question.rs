use super::Sentence;
use nar_dev_utils::join;

pub trait Question: Sentence {
    // ! ❌不能在此自动实现`isQuestion` `asQuestion`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法

    // ! ❌不能在此自动实现`toKey` `sentenceToString`
    // * 📝或者，Rust不允许类似「继承」的「实现一部分，丢给别的类型再实现另一部分」的做法
    /// 作为一个[`Sentence::to_key`]的默认【非覆盖性】实现
    fn question_to_key(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string()
        }
    }

    /// 作为一个[`Sentence::sentence_to_display`]的默认【非覆盖性】实现
    fn question_to_display(&self) -> String {
        join! {
            => self.content().to_string()
            => self.punctuation().to_string()
            => self.stamp_to_display()
        }
    }
}
