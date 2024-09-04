//! 存放与实际推理相关性不大、起支持性作用的类型、特征等

nar_dev_utils::mods! {
    // 可迭代对象
    pub use iterable;

    // 共享引用
    pub use rc;

    // 带序列号的共享引用
    pub use serial_rc;

    // 均值
    pub use average;

}

// 字符串呈现 | 内含导出的宏
mod to_display;
pub use to_display::*;

// 测试用 | 内含导出的宏
mod testing;
#[cfg(test)]
pub use testing::AResult; // ! 仅在测试中使用
