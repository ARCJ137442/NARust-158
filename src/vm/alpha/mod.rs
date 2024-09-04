//! NARust内置的「Alpha」型号 虚拟机

use nar_dev_utils::mods;

mods! {
    // 启动器
    pub use launcher;

    // 运行时
    pub use runtime;

    // 输入输出
    pub use io;

    // 指令分派
    pub use cmd_dispatch;
}
