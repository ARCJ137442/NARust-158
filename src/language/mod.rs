//! 语言
//! * 🎯有关Narsese的结构实现
//!
//! * 🚩【2024-04-21 02:06:58】目前实现了两个版本的Narsese
//!   * 📌一个基于「纯枚举」：每个「词项类型」都对应一个枚举项
//!   * 📌一个基于「标识符+容器」结构：更宽的范围，更通用的「词项类型」
//!     * 🌟目前将其作为「预备候选」留作继续开发
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

// 当前版本
mod term_impl;

// 对外公开的实现
pub use term_impl::*;
