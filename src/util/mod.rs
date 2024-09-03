//! 存放与实际推理相关性不大、起支持性作用的类型、特征等

nar_dev_utils::mods! {
    // 可迭代对象
    pub use iterable;

    // 引用/可空引用
    pub use option_or_some_ref;

    // 共享引用
    pub use rc;

    // 带序列号的共享引用
    pub use serial_rc;

    // 均值
    pub use average;

}
// 一次性实现
// TODO: 🏗️【2024-09-04 01:07:42】有待提取到`nar_dev_utils`中
mod impl_once;

// 字符串呈现 | 内含导出的宏
mod to_display;
pub use to_display::*;

// 测试用 | 内含导出的宏
mod testing;
#[cfg(test)]
pub use testing::AResult; // ! 仅在测试中使用
