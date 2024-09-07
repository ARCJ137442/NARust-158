//! 与其它类型相互转换
//! * 🎯转换为「词法Narsese」以便「获取名称」

// 其它类型 → 词项
mod from;

// 词项 → 其它类型（通用From/Into）
mod into;

// 词法折叠（词法Narsese→词项）
mod lexical_fold;
