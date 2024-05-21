//! 🆕有关「推导上下文」与「记忆区」的互操作
//! * 🎯分开存放[「记忆区」](crate::storage::Memory)中与「推导上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4
//!
//! * ♻️【2024-05-22 02:17:35】现经「大重构」后无甚保留代码，仅存OpenNARS中`make`系列方法

nar_dev_utils::mods! {
    // `make`系列方法，用于【预简化】词项
    pub use make_term;
}
