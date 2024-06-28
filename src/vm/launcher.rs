//! 虚拟机启动器
//! * 🚩【2024-05-15 10:35:59】启动器依赖运行时（及其实现）
//!
//! * ✅【2024-05-15 17:01:58】完成初代实现：名称、超参数

use super::Runtime;
use crate::{control::Parameters, inference::InferenceEngine};
use anyhow::Result;
use navm::vm::VmLauncher;

/// 虚拟机启动器
/// * 🎯作为启动虚拟机的配置与脚手架
#[derive(Debug, Clone)]
pub struct Launcher {
    /// 虚拟机名称
    /// * 🚩即「推理器名称」
    name: String,
    /// 超参数
    hyper_parameters: Parameters,
    /// 推理引擎
    inference_engine: InferenceEngine,
}

impl Launcher {
    /// 构造函数
    pub fn new(
        name: impl Into<String>,
        hyper_parameters: Parameters,
        inference_engine: InferenceEngine,
    ) -> Self {
        Self {
            name: name.into(),
            hyper_parameters,
            inference_engine,
        }
    }
}

/// 虚拟机启动器
impl VmLauncher for Launcher {
    type Runtime = Runtime;

    fn launch(self) -> Result<Self::Runtime> {
        // * 🚩创建新运行时
        let runtime = Runtime::new(self.name, self.hyper_parameters, self.inference_engine);
        // * 🚩返回
        Ok(runtime)
    }
}
