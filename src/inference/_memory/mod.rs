//! 🆕有关「推导上下文」与「记忆区」的互操作
//! * 🎯分开存放[「记忆区」](crate::storage::Memory)中与「推导上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4
//!
//! * ♻️【2024-05-16 17:57:40】现在将主要代码放在[`main`]子模块中

nar_dev_utils::mods! {
    // 推理循环
    pub use cycle_process;
    // 导出结论
    pub use derivation_process;
}
