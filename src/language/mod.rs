//! 语言
//! * 🎯有关Narsese的结构实现
//!
//! # 📄OpenNARS
//!
//! Term hierarchy in Narsese
//!
//! Open-NARS implements the following formal language, Narsese.
//!
//! ```ebnf
//!           <sentence> ::= <judgement>
//!                        | <question>
//!           <judgement> ::= <statement> <truth-value>
//!           <question> ::= <statement>
//!          <statement> ::= <<term> <relation> <term>>
//!                        | <compound-statement>
//!                        | <term>
//!               <term> ::= <word>
//!                        | <variable>
//!                        | <compound-term>
//!                        | <statement>
//!           <relation> ::= -->    // Inheritance
//!                        | <->    // Similarity
//!                        | {--    // Instance
//!                        | --]    // Property
//!                        | {-]    // InstanceProperty
//!                        | ==>    // Implication
//!                        | <=>    // Equivalence
//! <compound-statement> ::= (-- <statement>)                 // Negation
//!                        | (|| <statement> <statement>+)    // Disjunction
//!                        | (&& <statement> <statement>+)    // Conjunction
//!      <compound-term> ::= {<term>+}    // SetExt
//!                        | [<term>+]    // SetInt
//!                        | (& <term> <term>+)    // IntersectionExt
//!                        | (| <term> <term>+)    // IntersectionInt
//!                        | (- <term> <term>)     // DifferenceExt
//!                        | (~ <term> <term>)     // DifferenceInt
//!                        | (* <term> <term>+)    // Product
//!                        | (/ <term>+ _ <term>*)    // ImageExt
//!                        | (\ <term>+ _ <term>*)    // ImageInt
//!           <variable> ::= <independent-var>
//!                        | <dependent-var>
//!                        | <query-var>
//!    <independent-var> ::= $[<word>]
//!      <dependent-var> ::= #<word>
//!          <query-var> ::= ?[<word>]
//!               <word> : string in an alphabet
//!        <truth-value> : a pair of real numbers in [0, 1] x (0, 1)
//! ```
//!
//! Major methods in the Term classes:
//!
//! - constructors
//! - get and set
//! - clone, compare, and unify
//! - create and access corresponding concept
//! - structural operation in compound
//!
//! # logs
//!
//! ## language
//!
//! * 🚩【2024-04-21 02:06:58】目前实现了两个版本的Narsese
//!   * 📌一个基于「纯枚举」：每个「词项类型」都对应一个枚举项
//!   * 📌一个基于「标识符+容器」结构：更宽的范围，更通用的「词项类型」
//!     * 🌟目前将其作为「预备候选」留作继续开发
//! * 🚩【2024-09-07 16:19:17】默认此中「语言部分」仅有「词项定义」内容，并因此解包内部`term_impl`模块
//!
//! ## term_impl
//!
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

// 基础：结构、属性
mod base;
pub use base::*;

// 转换
mod conversion;

// 【内建】方言（解析器）
#[cfg(feature = "dialect_parser")]
pub mod dialect;

// 各词项基于改版源码的「特性」
mod features;
pub use features::*;

// 「变量处理」
pub mod variable_process;
