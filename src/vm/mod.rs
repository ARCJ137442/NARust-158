//! 🆕NARust的NAVM接口
//! * 🎯接入NAVM，在源码层实现统一输入输出

// 启动器
mod launcher;
pub use launcher::*;

// 运行时
mod runtime;
pub use runtime::*;
