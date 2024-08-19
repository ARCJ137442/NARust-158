//! 🎯复刻OpenNARS改版 `nars.control.Reasoner`
//! * 🆕此类的实现有相对较多的个人化写法
//!   * 模块拆分、函数重组 等……
//!
//! * ♻️【2024-06-26 11:48:57】目前开始着手按OpenNARS改版重写

nar_dev_utils::mods! {
    // 定义
    pub use definition;

    // 功能：推理数据
    pub use derivation_datas;

    // 功能：输出报告
    pub use report;

    // 功能：序列反序列化
    pub use serde;

    // 功能：NAVM接口
    pub use vm_api;
}
