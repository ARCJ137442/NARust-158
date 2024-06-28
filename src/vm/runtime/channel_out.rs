//! 为推理器新实现的「输出通道」
//! * 💫会不会导致循环引用问题？运行时→推理器→通道→运行时
//!   * 💭【2024-05-15 10:55:56】一个方案
//!     * 🚩通道以`Rc<RefCell>`在运行时、推理器中存在两个备份
//!     * 🚩通道自身保存一个「缓存的输出」
//!       * 🚩被推理器调用时，存入输出
//!       * 🚩运行时被拉取输出时，从中拉取
//!     * ✅单线程不会导致借用问题

use crate::{
    global::RC,
    io::{Channel, OutputChannel},
    util::RefCount,
};
use navm::output::Output;
use std::collections::VecDeque;

/// 初代通用`OutputChannel`实现
#[derive(Debug, Clone, Default)]
pub struct ChannelOut {
    /// 缓存的输出
    cached_outputs: VecDeque<Output>,
}

impl ChannelOut {
    /// 构造函数
    pub fn new() -> Self {
        Self {
            cached_outputs: VecDeque::new(),
        }
    }

    /// 拉取缓存的输出
    /// * 🚩先进先出
    pub fn fetch(&mut self) -> Option<Output> {
        self.cached_outputs.pop_front()
    }

    /// 从「共享引用」中拉取缓存的输出
    /// * 🚩先进先出
    /// * 🚩【2024-05-15 11:16:05】对错误采取「打印错误并失败」的处理方法
    pub fn fetch_rc(this: &mut RC<Self>) -> Option<Output> {
        this.mut_().fetch()
    }
}

impl Channel for ChannelOut {
    /// 始终无需移除
    fn need_remove(&self) -> bool {
        false
    }
}

/// 对自身实现
impl OutputChannel for ChannelOut {
    fn next_output(&mut self, outputs: &[Output]) {
        // * 🚩（复制并）存入自身缓存中
        self.cached_outputs.extend(outputs.iter().cloned());
    }
}

impl Channel for RC<ChannelOut> {
    /// 委托到内部值
    fn need_remove(&self) -> bool {
        self.get_().need_remove()
    }
}

/// 对Rc<RefCell>自身实现
impl OutputChannel for RC<ChannelOut> {
    fn next_output(&mut self, outputs: &[Output]) {
        self.mut_().next_output(outputs)
    }
}
