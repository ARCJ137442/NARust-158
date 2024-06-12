//! 表征NARust 158所用的「词项」
//! * 📄功能上参照OpenNARS
//! * 🚩实现方式上更Rusty，同时亦有其它妥协/加强
//! * ❓【2024-04-20 22:00:44】「统一结构体+用『可选字段』实现多态」的方法，会导致「性能臃肿」问题
//!   * ❗此举需要提前考虑「所有类型词项的所有功能」，并且要做到最大程度兼容
//!   * 📌即便使用「作为枚举的专用字段」也会因为「要适应某种复合词项类型」而导致让步
//!     * 而这种「只会在某个类型上产生让步」的方法，会导致「本该耦合而未耦合」的情形
//!     * 这种「看似通用，实则仍需『专用情况专用对待』」的方法，不利于后续维护
//!   * ❓【2024-04-20 23:53:15】或许也可行：是否可以`match (self.identifier, &*self.components)`
//! * 🚩【2024-04-20 22:05:09】目前将此方案搁置
//!   * ⇒尝试探索「直接基于『枚举Narsese』」的方法
//! * 🚩【2024-04-25 08:36:07】在`term_v3`、`term_v4`相继失败后，重启该方法
//!   * 📌通过「限制构造函数」+「只处理特定词项模式」的方法，基本解决堵点

use crate::io::symbols::*; // ! 📌【2024-04-25 23:37:20】这些在各大子模块的`use super::*`中用到
use nar_dev_utils::manipulate;

// 结构
mod structs;
pub use structs::*;

// 实现 / 构造
mod construct;

// 【内建】与其它类型相互转换
mod _conversion;

// 【内建】方言解析器
#[cfg(feature = "dialect_parser")]
pub mod _dialect;
#[cfg(feature = "dialect_parser")]
pub use _dialect as dialect;

// 【内建】实现 / 属性
mod _property;

// 📄OpenNARS `nars.language.Term`
mod term;

// 📄OpenNARS `nars.language.CompoundTerm`
mod compound;

// 📄OpenNARS `nars.language.Variable`
pub mod variable;

// 📄OpenNARS `nars.language.Statement`
mod statement;

// 📄OpenNARS `nars.language.ImageXXt`
mod image;
