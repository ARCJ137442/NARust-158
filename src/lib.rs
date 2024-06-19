//! NARust-158 主模块

// 实用
pub mod util;

// 全局
pub mod global;

// 语言
pub mod language;

// 输入输出
pub mod io;

// // 实体
// pub mod entity;

// 存储
pub mod storage;

// // 推理
// pub mod inference;

// // 控制
// pub mod control;

// 虚拟机
pub mod vm;

// 「主」模块（📄OpenNARS）
// * ⚠️【2024-04-27 11:42:28】不建议用`main`作为模块名
//   * 📄 "found module declaration for main.rs, a binary crate cannot be used as library"
// * 🆕修改模块名`main` => `nars`
pub mod nars;
