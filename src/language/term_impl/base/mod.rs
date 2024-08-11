//! Narsese结构 基础
//! * 🎯基础数据结构定义
//! * 🎯数据结构基本属性、构造、转换 API

// 结构
mod structs;
pub use structs::*;

// 实现 / 构造
mod construct;

// 【内建】实现 / 属性
mod property;

// 【内建】与其它类型相互转换
mod conversion;

// 【对外】序列反序列化
mod serde;
