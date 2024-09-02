//! 🎯复刻OpenNARS `nars.entity.Item`
//! * ✅【2024-05-02 00:54:15】所有方法基本复刻完毕

use super::BudgetValue;
use crate::{inference::Budget, util::ToDisplayAndBrief};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

/// 与「键」有关的特征
/// * 🎯基本要求：可被索引（[`Hash`]）
///   * 用于散列映射
/// * 📌【2024-09-02 22:24:58】其它特征约束仅为与字符串对齐
///   * TODO: 解耦字符串相关方法，并减少约束量
pub trait ItemKey:
    Debug + Clone + Default + Eq + Ord + Hash + Serialize + DeserializeOwned
{
}
impl ItemKey for String {}

/// 模拟`nars.entity.Item`
/// * 📌袋中的「物品」类型
/// * 📝实际上其「键」和其「预算」都应只限于在「袋」内
///   * 📌即便实际上其自身有存储，也不过是在一种「特殊条件」下进行
/// * 🚩【2024-04-28 08:38:15】目前仍然先参照OpenNARS的方法来
///   * 在`Item`类中，有存在「不通过『袋』访问『预算』」的情况
/// * 🚩【2024-05-01 23:17:26】暂且按照OpenNARS的命名来：直接使用`Item`而非`BagItem`
/// * 📍示例：只实现「像预算那样『具有p、d、q属性』，但不仅仅有p、d、q属性」且不能直接从p、d、q构造
///   * ℹ️亦即：实现[`BudgetValue`]而未实现[`BudgetValueConcrete`]
///   * ✨并不「继承」预算值，但却可以当预算值一样使用（属性&方法）
///
/// An item is an object that can be put into a Bag,
/// to participate in the resource competition of the system.
///
/// It has a key and a budget. Cannot be cloned
pub trait Item: Budget {
    /// 绑定的「键」类型
    type Key: ItemKey;
    /// 获取其元素id
    /// * 🎯应该只与自身数据绑定
    ///   * 📄概念的「词项名」
    fn key(&self) -> &Self::Key;

    /// 🆕决定「袋」中俩[`Item`]的合并顺序
    /// * 📜默认的「合并顺序」：旧→新（然后抛掉旧的）
    fn merge_order(&self, _new: &Self) -> MergeOrder {
        MergeOrder::OldToNew
    }
}

/// 预算合并顺序（枚举）
/// * 🎯用于「袋」中「相同key值预算该从哪儿合并到哪儿」的逻辑
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeOrder {
    /// 从「将移出的Item」合并到「新进入的Item」
    /// * 📌修改「新进入的Item」
    OldToNew,
    /// 从「新进入的Item」合并到「将移出的Item」
    /// * 📌修改「将移出的Item」
    NewToOld,
}

/// 🆕一个基于「复合」而非「继承」的[`Item`]默认实现
/// * 🎯用于内含字段并让「任务」「概念」等分发
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    /// The key of the Item, unique in a Bag
    /// * ❓后续可以放入「袋」中，使用「Key → Item(T, Budget)」的结构将「预算值」完全合并入「袋」中
    ///   * 📌【2024-06-21 22:34:13】注：这是个大工程，需要完全不同的数据类型架构
    ///   * 📄参考OpenNARS改版的`dev-bag-item`分支
    key: String,

    /// The budget of the Item, consisting of 3 numbers
    /// * 📝仅用于各预算值函数，以及在「袋」中的选取（优先级）
    budget: BudgetValue,
}

impl Token {
    /// 构造函数
    /// * 📌对所有参数均要求完全所有（排避免意外的共享引用）
    pub fn new(key: impl Into<String>, budget: BudgetValue) -> Self {
        Token {
            key: key.into(),
            budget,
        }
    }

    /// 预算值（读写）
    pub fn budget(&self) -> &BudgetValue {
        &self.budget
    }

    /// 预算值（读写）
    pub fn budget_mut(&mut self) -> &mut BudgetValue {
        &mut self.budget
    }
}

impl ToDisplayAndBrief for Token {
    fn to_display(&self) -> String {
        format!("{} {}", self.budget_to_display(), self.key)
    }

    fn to_display_brief(&self) -> String {
        format!("{} {}", self.budget_to_display_brief(), self.key)
    }
}

// 委托实现「预算值」
impl Budget for Token {
    fn priority(&self) -> super::ShortFloat {
        self.budget.priority()
    }

    fn __priority_mut(&mut self) -> &mut super::ShortFloat {
        self.budget.__priority_mut()
    }

    fn durability(&self) -> super::ShortFloat {
        self.budget.durability()
    }

    fn __durability_mut(&mut self) -> &mut super::ShortFloat {
        self.budget.__durability_mut()
    }

    fn quality(&self) -> super::ShortFloat {
        self.budget.quality()
    }

    fn __quality_mut(&mut self) -> &mut super::ShortFloat {
        self.budget.__quality_mut()
    }
}

impl Item for Token {
    type Key = String;
    /// 键（只读）
    fn key(&self) -> &String {
        &self.key
    }
}
