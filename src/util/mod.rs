//! 存放与实际推理相关性不大、起支持性作用的类型、特征等

nar_dev_utils::mods! {
    // 可迭代对象
    pub use iterable;

    // 字符串呈现
    pub use to_display;

    // 共享引用
    pub use rc;
}

// 测试用
// * ❌【2024-06-20 02:02:25】莫尝试「模块封装+自动导出」省`test::`
//   * ⚠️报警：`private item shadows public glob re-export`

/// 测试用类型，增强[`anyhow::Result`]
#[cfg(test)]
pub type AResult<T = ()> = anyhow::Result<T>;

/// 测试用宏，简化`Ok(())`
#[cfg(test)]
#[macro_export]
macro_rules! ok {
    () => {
        Ok(())
    };
    ($($code:tt)*) => {
        Ok($($code)*)
    };
}
