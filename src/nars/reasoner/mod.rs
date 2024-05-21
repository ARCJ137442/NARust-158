//! 🎯复刻OpenNARS `nars.main_nogui.ReasonerBatch`
//! * 🆕此类的实现有相对较多的个人化写法
//!   * 模块拆分、函数重组 等……

nar_dev_utils::mods! {
    // 定义
    pub use definition;
    // 功能：解析Narsese任务
    pub use parse_task;
    // 功能：输出报告
    pub use report;
}
