//! 为推理器新实现的「输入通道」
use super::*;
use crate::{
    io::{Channel, InputChannel},
    util::RefCount,
};
use navm::cmd::Cmd;

/// 初代通用`InputChannel`实现
/// * 🚩【2024-05-17 17:01:54】没有「初代输入通道」：暂时不需要
/// * 🚩【2024-06-29 00:47:31】现在需要「初代输入通道」
///   * 🎯检验并利用推理器自身机制
///   * ✅技术不难：通过函数指针很轻松地引入外部代码
#[derive(Debug, Clone)]
pub struct ChannelIn {
    /// 输入源（一个）
    /// * 🚩可返回指令，亦可不返回指令
    input_source: fn() -> Option<Cmd>,
}

impl ChannelIn {
    /// 构造函数
    pub fn new(input_source: fn() -> Option<Cmd>) -> Self {
        Self { input_source }
    }

    /// 拉取输入
    /// * 🚩先进先出
    pub fn fetch(&self) -> Option<Cmd> {
        (self.input_source)()
    }

    /// 从「共享引用」中拉取输入
    #[inline]
    pub fn fetch_rc(this: &mut RC<Self>) -> Option<Cmd> {
        this.get_().fetch()
    }
}

impl Channel for ChannelIn {
    /// 始终无需移除
    fn need_remove(&self) -> bool {
        false
    }
}

/// 对自身实现
impl InputChannel for ChannelIn {
    fn next_input(&mut self) -> (bool, Vec<Cmd>) {
        // * 🚩拉取自身输出，并通过`is_some`决定「是否阻塞推理器」
        let cmd = self.fetch();
        (cmd.is_some(), cmd.into_iter().collect())
    }
}

impl Channel for RC<ChannelIn> {
    /// 委托到内部值
    fn need_remove(&self) -> bool {
        self.get_().need_remove()
    }
}

/// 对Rc<RefCell>自身实现
impl InputChannel for RC<ChannelIn> {
    fn next_input(&mut self) -> (bool, Vec<Cmd>) {
        self.mut_().next_input()
    }
}
