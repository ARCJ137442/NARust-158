//! 🆕存放有关「推理处理」的函数
//! * 📄直接推理
//! * 📄概念推理（&转换推理）
//!
//! 原`global`模块
//! * 📌推理工作周期入口
//! * 📌具体推理过程组织：直接推理⇒转换推理⇒概念推理
//!
//! * ♻️【2024-06-26 11:50:22】目前开始着手按改版OpenNARS重写

nar_dev_utils::mods! {
    // 工作周期
    pub use work_cycle;

    // 直接推理 | 以原OpenNARS 1.5.8`Concept.directProcess`命名
    pub use process_direct;

    // 概念推理 | 以原OpenNARS 1.5.8`RuleTables.reason`命名
    pub use process_reason;
}
