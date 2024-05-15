//! 虚拟机启动器
//! * 🚩【2024-05-15 10:35:59】启动器依赖运行时（及其实现）
//!
//! TODO: 【2024-05-15 10:37:42】随「推理器」完善后修改、润色、完善

use super::Runtime;
use crate::nars::{Parameters, ReasonerConcrete};
use anyhow::Result;
use navm::vm::VmLauncher;
use std::marker::PhantomData;

/// 虚拟机启动器
/// * 🎯作为启动虚拟机的配置与脚手架
///
/// TODO: 🏗️后续可引入诸如「启动参数」等
pub struct Launcher<R: ReasonerConcrete> {
    /// 类型标注
    _marker: PhantomData<R>,
    /// 超参数
    hyper_parameters: Parameters,
}

impl<R: ReasonerConcrete> Launcher<R> {
    /// 构造函数
    pub fn new(hyper_parameters: Parameters) -> Self {
        Self {
            _marker: PhantomData,
            hyper_parameters,
        }
    }
}

/// 虚拟机启动器
impl<R: ReasonerConcrete> VmLauncher for Launcher<R> {
    type Runtime = Runtime<R>;

    fn launch(self) -> Result<Self::Runtime> {
        // * 🚩创建新运行时
        let runtime = Runtime::new(self.hyper_parameters);
        // * 🚩返回
        Ok(runtime)
    }
}
