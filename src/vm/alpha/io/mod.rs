//! 有关内置虚拟机「Alpha」的输入输出功能
//!
//! # 原`src/io`注释
//!
//! 输入输出
//! * 🎯有关NARS输入输出方面的功能
//!
//! # 📄OpenNARS
//!
//! Input/output management
//!
//! All Narsese-based input/output interfaces of the system are defined in this package.

use nar_dev_utils::mods;

mods! {
    // 处理输入输出
    pub use handle_io;

    // 🆕通道
    pub use _channel;

    // 输入通道
    pub use input_channel;

    // 输出通道
    pub use output_channel;

    // 输入通道实现
    pub use channel_in;

    // 输出通道实现
    pub use channel_out;

    // IO通道 数据结构
    pub use channels;
}
