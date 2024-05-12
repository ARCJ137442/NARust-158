//! 🎯复刻OpenNARS `nars.io.InputChannel`
//!

use super::Channel;
use navm::cmd::Cmd;

/// 模拟`InputChannel`
/// * 🆕✨采用「返回NAVM指令」的形式
///   * 💭【2024-05-13 00:55:15】此举相当于在编程上统一了IO模型
///
/// # 📄OpenNARS
///
/// An interface to be implemented in all input channels
/// to get the input for the next moment from an input channel
///
/// # 用例
///
/// ```rust
/// use narust_158::io::{Channel, InputChannel};
/// use navm::cmd::Cmd;
///
/// pub struct C;
/// impl Channel for C {
///     fn need_remove(&self) -> bool {
///         false
///     }
/// }
/// impl InputChannel for C {
///     fn next_input(&mut self) -> (bool, Vec<Cmd>) {
///         (true, vec![])
///     }
/// }
///
/// let mut c = C;
/// assert_eq!(c.next_input(), (true, vec![]));
/// assert!(!c.need_remove());
/// let dyn_c: &mut dyn InputChannel = &mut c;
/// dyn_c.next_input();
/// assert_eq!(dyn_c.next_input(), (true, vec![]));
/// assert!(!dyn_c.need_remove()); // 变为动态引用之后，具体类型被抹除，但超特征方法仍然可以引用
/// let mut box_c: Box<dyn InputChannel> = Box::new(c);
/// box_c.next_input();
/// assert_eq!(box_c.next_input(), (true, vec![]));
/// assert!(!box_c.need_remove()); // 变为「装箱的特征对象」也一样
/// ```
pub trait InputChannel: Channel {
    /// 模拟`InputChannel.nextInput`
    /// * ⚠️看似「不可变」，实际上**有副作用**
    ///   * 📝OpenNARS中的实现[`ExperienceReader`]持有推理器引用，会由此改变推理器
    ///   * ❓【2024-05-13 00:01:07】后续是否要改变这种模式
    /// * 🆕🚩鉴于OpenNARS中「请求输入」的作用，现消去其对推理器的副作用
    ///   * 📌对推理器的「输入呈递」从「自推理器（循环）引用直接传递」改为「函数返回值」
    ///   * 📌这个「呈递的输入」以「[NAVM指令](navm::cmd::Cmd)数组」的形式给出
    /// * 🆕引入新的「推理器」参数（只读），
    ///   * 🎯以便后续在「解析生成NAVM指令」时结合「推理器状态」与「记忆区」
    ///
    /// # 📄OpenNARS
    ///
    /// @return value indicating whether the reasoner should run
    fn next_input(&mut self, reasoner: &Self::Reasoner) -> (bool, Vec<Cmd>);
}
