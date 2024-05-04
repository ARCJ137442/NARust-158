//! 存储一些「全局」参数
//! * 🎯参数类型如「浮点数」（选择哪个精度）
//! * ⚠️【2024-04-27 10:47:59】尽量不要用来存储常量

/// 全局浮点数类型
pub type Float = f64;

/// 全局引用计数类型
/// * 🚩【2024-05-04 16:51:11】目前尚未做多线程兼容，此处仅考虑单线程，
///   * 故暂且使用 [`std::rc::Rc`] 而非 [`std::sync::Arc`]
pub type RC<T> = std::rc::Rc<T>;

/// 全局「时钟数」类型
/// * 🎯NARS内部推理时间
/// * 🎯时间戳[`crate::entity::Stamp`]
/// * 🚩【2024-05-04 17:41:49】目前设定为无符号整数，对标OpenNARS中的`long`长整数类型
///   * 📝OpenNARS中也是将其作为无符号整数（非负整数）用的
pub type ClockTime = usize;

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
