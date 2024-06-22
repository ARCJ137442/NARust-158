//! 有关「袋」的数据结构定义

// 特征
mod traits;
pub use traits::*;

// 【内部】分派器
mod distributor;
use distributor::*;

// 【内部】表
mod impl_tables;
use impl_tables::*;

// 初代实现
mod impl_v1;
pub use impl_v1::*;
