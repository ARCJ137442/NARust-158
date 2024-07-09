//! 存放与实际推理相关性不大、起支持性作用的类型、特征等

nar_dev_utils::mods! {
    // 可迭代对象
    pub use iterable;

    // 引用/可空引用
    pub use option_or_some_ref;

    // 共享引用
    pub use rc;
}

// 字符串呈现 | 内含导出的宏
mod to_display;
pub use to_display::*;

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

/// 测试用宏，用于简化调试模式断言
#[macro_export]
macro_rules! debug_assert_matches {
    ($value:expr, $pattern:pat $(, $($tail:tt)*)?) => {
        debug_assert!(matches!($value, $pattern) $(, $($tail)*)?)
    };
}
