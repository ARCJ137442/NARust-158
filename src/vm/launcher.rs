//! 虚拟机启动器
//! * 🚩【2024-05-15 10:35:59】启动器依赖运行时（及其实现）
//!
//! * ✅【2024-05-15 17:01:58】完成初代实现：名称、超参数

use super::Runtime;
use crate::nars::{Parameters, ReasonerConcrete};
use anyhow::Result;
use navm::vm::VmLauncher;
use std::marker::PhantomData;

/// 虚拟机启动器
/// * 🎯作为启动虚拟机的配置与脚手架
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Launcher<R: ReasonerConcrete> {
    /// 「推理器」类型标注`R`
    _marker: PhantomData<R>,
    /// 虚拟机名称
    /// * 🚩即「推理器名称」
    name: String,
    /// 超参数
    hyper_parameters: Parameters,
}

impl<R: ReasonerConcrete> Launcher<R> {
    /// 构造函数
    pub fn new(name: impl Into<String>, hyper_parameters: Parameters) -> Self {
        Self {
            _marker: PhantomData,
            name: name.into(),
            hyper_parameters,
        }
    }
}

/// 虚拟机启动器
impl<R: ReasonerConcrete> VmLauncher for Launcher<R> {
    type Runtime = Runtime<R>;

    fn launch(self) -> Result<Self::Runtime> {
        // * 🚩创建新运行时
        let runtime = Runtime::new(self.name, self.hyper_parameters);
        // * 🚩返回
        Ok(runtime)
    }
}
