//! 🎯复刻OpenNARS `nars.io.OutputChannel`
//!

use super::Channel;
use navm::output::Output;

/// 模拟`OutputChannel`
///
/// # 📄OpenNARS
///
/// An interface to be implemented in all output channel
///
/// # 用例
///
/// ```rust
/// use narust_158::io::{Channel, OutputChannel};
/// use navm::output::Output;
///
/// pub struct C;
/// impl Channel for C {
///     fn need_remove(&self) -> bool {
///         false
///     }
/// }
/// impl OutputChannel for C {
///     fn next_output(&mut self, outputs: &[Output]) {
///         println!("{:?}", outputs);
///     }
/// }
///
/// let mut c = C;
/// let o = vec![Output::COMMENT { content: "out".to_string() }];
/// c.next_output(&o);
/// assert!(!c.need_remove());
/// let dyn_c: &mut dyn OutputChannel = &mut c;
/// dyn_c.next_output(&o);
/// assert!(!dyn_c.need_remove()); // 变为动态引用之后，具体类型被抹除，但超特征方法仍然可以引用
/// let mut box_c: Box<dyn OutputChannel> = Box::new(c);
/// box_c.next_output(&o);
/// assert!(!box_c.need_remove()); // 变为「装箱的特征对象」也一样
/// ```
pub trait OutputChannel: Channel {
    /// 模拟`OutputChannel.nextOutput`
    /// * ⚠️看似「不可变」，实际上**有副作用**
    ///   * 📝实际逻辑是「接收NARS的输出，并自行处理」
    ///   * ❓【2024-05-13 00:01:07】后续是否要改变这种模式
    /// * 🆕🚩鉴于在OpenNARS中对相应实现的观察，现将其中的「字符串」改为「NAVM输出」
    ///   * 💭【2024-05-13 00:57:42】这可能跟「NAVM模型」中定义的「输出缓冲区」不一样——有多个，而非仅从一个之中拉取
    /// * 🆕引入新的「推理器」参数，
    ///   * 🎯以便后续在「解读NAVM输出」时结合「推理器状态」与「记忆区」
    ///   * 🚩【2024-05-13 10:48:14】现在让「推理器」可写，以便后续反向控制推理器
    ///     * ✅【2024-05-13 10:48:46】保证可行性，但后续可能会有安全问题
    ///     * ❓到底应不应该「反向修改推理器」
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn next_output(&mut self, reasoner: &mut Self::Reasoner, outputs: &[Output]);

    // ! ❌【2024-05-13 00:02:26】暂不实现`tickTimer`呈现用函数
}
