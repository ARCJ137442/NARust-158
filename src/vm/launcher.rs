//! 虚拟机启动器
//! * 🚩【2024-05-15 10:35:59】启动器依赖运行时（及其实现）
//!
//! * ✅【2024-05-15 17:01:58】完成初代实现：名称、超参数

use super::Runtime;
use crate::{
    control::{Parameters, ReasonerConcrete},
    types::TypeContext,
};
use anyhow::Result;
use navm::vm::VmLauncher;
use std::marker::PhantomData;

/// 虚拟机启动器
/// * 🎯作为启动虚拟机的配置与脚手架
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Launcher<C, R>
where
    C: TypeContext,
    R: ReasonerConcrete<C>,
{
    /// 「推理器」类型标注`R`
    _marker_r: PhantomData<R>,
    /// 「推理器」类型标注`C`
    _marker_c: PhantomData<C>,
    /// 虚拟机名称
    /// * 🚩即「推理器名称」
    name: String,
    /// 超参数
    hyper_parameters: Parameters,
}

impl<C: TypeContext, R: ReasonerConcrete<C>> Launcher<C, R> {
    /// 构造函数
    pub fn new(name: impl Into<String>, hyper_parameters: Parameters) -> Self {
        Self {
            _marker_c: PhantomData,
            _marker_r: PhantomData,
            name: name.into(),
            hyper_parameters,
        }
    }
}

/// 虚拟机启动器
impl<C: TypeContext, R: ReasonerConcrete<C>> VmLauncher for Launcher<C, R> {
    type Runtime = Runtime<C, R>;

    fn launch(self) -> Result<Self::Runtime> {
        // * 🚩创建新运行时
        let runtime = Runtime::new(self.name, self.hyper_parameters);
        // * 🚩返回
        Ok(runtime)
    }
}
