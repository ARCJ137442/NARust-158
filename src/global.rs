//! 存储一些「全局」参数
//! * 🎯参数类型如「浮点数」（选择哪个精度）
//! * ⚠️【2024-04-27 10:47:59】尽量不要用来存储常量

pub type Float = f64;

/// 测试用
#[cfg(test)]
pub mod tests {
    /// 测试用类型，增强[`anyhow::Result`]
    pub type AResult<T = ()> = anyhow::Result<T>;

    /// 测试用宏，简化`Ok(())`
    #[macro_export]
    macro_rules! ok {
        () => {
            Ok(())
        };
        ($($code:tt)*) => {
            Ok($($code)*)
        };
    }
}
