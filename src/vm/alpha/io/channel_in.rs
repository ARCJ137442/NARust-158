//! 为推理器新实现的「输入通道」

use crate::{
    global::RC,
    vm::alpha::io::{Channel, InputChannel},
};
use nar_dev_utils::RefCount;
use navm::cmd::Cmd;
use std::collections::VecDeque;

/// 初代通用`InputChannel`实现
/// * 🚩【2024-05-17 17:01:54】没有「初代输入通道」：暂时不需要
/// * 🚩【2024-06-29 00:47:31】现在需要「初代输入通道」
///   * 🎯检验并利用推理器自身机制
///   * ✅技术不难：通过函数指针很轻松地引入外部代码
/// * 🚩【2024-06-29 01:14:48】现在基于外部需要，改为「虚拟机的输入在此临时存储」
#[derive(Debug, Clone, Default)]
pub struct ChannelIn {
    /// 缓存的输入
    cached_inputs: VecDeque<Cmd>,
}

impl ChannelIn {
    /// 构造函数
    /// * 🚩默认构造一个空通道
    pub fn new() -> Self {
        Self::default()
    }

    /// 放置输入
    /// * 🎯从NAVM虚拟机中放置，后续预计将被推理器自身拿出
    /// * 🚩先进先出
    pub fn put(&mut self, cmd: Cmd) {
        self.cached_inputs.push_back(cmd);
    }

    /// 向「共享引用」中放置输入
    #[inline]
    pub fn put_rc(this: &mut RC<Self>, cmd: Cmd) {
        this.mut_().put(cmd);
    }

    /// 拉取输入
    /// * 🚩先进先出
    pub fn fetch(&mut self) -> Option<Cmd> {
        self.cached_inputs.pop_front()
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
