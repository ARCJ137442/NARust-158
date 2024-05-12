//! 输入输出
//! * 🎯有关NARS输入输出方面的功能
//!
//! # 📄OpenNARS
//!
//! Input/output management
//!
//! All Narsese-based input/output interfaces of the system are defined in this package.

nar_dev_utils::mods! {
    // 符号 | 📌【2024-05-13 00:05:34】此处是特例：常量过多，需要封装
    use pub symbols;
    // 🆕通道
    pub use _channel;
    // 输入通道
    pub use input_channel;
    // 输出通道
    pub use output_channel;
}
