//! 🆕有关「推理上下文」与「概念」的互操作
//! * 🎯分开存放[「概念」](crate::entity::Concept)中与「推理上下文」有关的方法
//! * 📄仿自OpenNARS 3.0.4

nar_dev_utils::mods! {
    // 主模块
    pub use main;
    // 链接处理
    pub use link_process;
}
