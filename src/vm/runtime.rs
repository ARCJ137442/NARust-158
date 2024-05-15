//! 虚拟机运行时
//!
//! * ✅【2024-05-15 16:57:37】初代全功能实现

use crate::{
    global::{GlobalRc, GlobalRcMut, RCMut},
    nars::{Parameters, Reasoner, ReasonerConcrete},
};
use anyhow::Result;
use navm::{cmd::Cmd, output::Output, vm::VmRuntime};

/// 虚拟机运行时
/// * 🎯包装一个虚拟机，以跳出孤儿规则的限制
#[derive(Debug, Clone)]
pub struct Runtime<R: ReasonerConcrete> {
    /// 内部推理器字段
    reasoner: R,
    /// 输出通道的共享引用
    /// * 🎯避免「运行时→推理器→通道→运行时」的循环引用
    /// * 🚩「缓存的输出」亦包含在内
    o_channel: RCMut<ChannelOut<R>>,
}

/// 自身实现
impl<'this: 'reasoner, 'reasoner, R: ReasonerConcrete + 'reasoner> Runtime<R>
where
    Self: 'this,
{
    /// 构造函数
    /// * 🚩【2024-05-15 10:40:49】暂不允许「直接由推理器创建」
    ///   * 📌需要更精细地控制「内部推理器」的状态与成员
    pub fn new(name: impl Into<String>, hyper_parameters: Parameters) -> Self {
        // * 🚩创建推理器
        let mut reasoner = R::__new(name.into(), hyper_parameters);

        // * 🚩创建并加入通道
        let o_channel = RCMut::new_(ChannelOut::new());
        let b = Box::new(o_channel.clone());
        reasoner.add_output_channel(b); // * ✅解决：在「推理器」中细化生命周期约束，现在不再报错与要求`'static`

        // * 🚩构造自身
        Self {
            // * 🚩载入推理器
            reasoner,
            // * 🚩空通道
            o_channel,
        }
    }
}

/// 实现[虚拟机运行时](VmRuntime)
impl<R: ReasonerConcrete> VmRuntime for Runtime<R> {
    fn input_cmd(&mut self, cmd: Cmd) -> Result<()> {
        Reasoner::input_cmd(&mut self.reasoner, cmd);
        Ok(())
    }

    fn fetch_output(&mut self) -> Result<Output> {
        self.o_channel
            .mut_()
            .fetch()
            .ok_or(anyhow::anyhow!("没有输出"))
    }

    fn try_fetch_output(&mut self) -> Result<Option<Output>> {
        Ok(self.o_channel.mut_().fetch())
    }

    fn status(&self) -> &navm::vm::VmStatus {
        // * 🚩【2024-05-15 16:39:12】始终在运行
        // * ❓貌似Rust版本并不一定要像Java版本那样区分「在运行」与「不在运行」——随时输入随时处理
        &navm::vm::VmStatus::Running
    }

    fn terminate(&mut self) -> Result<()> {
        // * 🚩重置推理器
        self.reasoner.reset();
        // * 🚩返回
        Ok(())
    }
}

/// 为推理器新实现的「通道」
/// * 💫会不会导致循环引用问题？运行时→推理器→通道→运行时
///   * 💭【2024-05-15 10:55:56】一个方案
///     * 🚩通道以`Rc<RefCell>`在运行时、推理器中存在两个备份
///     * 🚩通道自身保存一个「缓存的输出」
///       * 🚩被推理器调用时，存入输出
///       * 🚩运行时被拉取输出时，从中拉取
///     * ✅单线程不会导致借用问题
mod channels {
    use super::*;
    use crate::io::{Channel, OutputChannel};
    use std::collections::VecDeque;

    #[derive(Debug, Clone)]
    pub struct ChannelOut<R: ReasonerConcrete> {
        _marker: std::marker::PhantomData<R>,
        cached_outputs: VecDeque<Output>,
    }

    impl<R: ReasonerConcrete> ChannelOut<R> {
        /// 构造函数
        pub fn new() -> Self {
            Self {
                _marker: std::marker::PhantomData,
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
        pub fn fetch_rc(this: &mut RCMut<Self>) -> Option<Output> {
            this.mut_().fetch()
        }
    }

    impl<R: ReasonerConcrete> Default for ChannelOut<R> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<R: ReasonerConcrete> Channel for ChannelOut<R> {
        type Reasoner = R;

        /// 始终无需移除
        fn need_remove(&self) -> bool {
            false
        }
    }

    /// 对自身实现
    impl<R: ReasonerConcrete> OutputChannel for ChannelOut<R> {
        fn next_output(&mut self, _reasoner: &mut Self::Reasoner, outputs: &[Output]) {
            // * 🚩（复制并）存入自身缓存中
            self.cached_outputs.extend(outputs.iter().cloned());
        }
    }

    impl<R: ReasonerConcrete> Channel for RCMut<ChannelOut<R>> {
        type Reasoner = R;

        /// 委托到内部值
        fn need_remove(&self) -> bool {
            self.get_().need_remove()
        }
    }

    /// 对Rc<RefCell>自身实现
    impl<R: ReasonerConcrete> OutputChannel for RCMut<ChannelOut<R>> {
        fn next_output(&mut self, reasoner: &mut Self::Reasoner, outputs: &[Output]) {
            self.mut_().next_output(reasoner, outputs)
            // match self.mut_() {
            //     Some(channel) => channel.next_output(reasoner, outputs),
            //     None => eprintln!("ChannelOut<R> is not initialized | outputs = {outputs:?}"),
            // }
        }
    }
}
pub use channels::*;
