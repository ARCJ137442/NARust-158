//! 实现 / 属性（内建）
//! * 🎯非OpenNARS所定义之「属性」「方法」
//!   * 📌至少并非OpenNARS原先所定义的

use super::*;

/// 手动实现「判等」逻辑
/// * 📄OpenNARS `Term.equals` 方法
/// * 🎯不让判等受各类「临时变量/词法无关的状态变量」的影响
///   * 📄`is_constant`字段
impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        /// 宏：逐个字段比较相等
        /// * 🎯方便表示、修改「要判等的字段」
        macro_rules! eq_fields {
            ($this:ident => $other:ident; $($field:ident)*) => {
                $( $this.$field == $other.$field )&&*
            };
        }
        // 判等逻辑
        eq_fields! {
            self => other;
            identifier
            components
        }
    }
}

/// 手动实现「散列」逻辑
/// * 🎯在手动实现「判等」后，无法自动实现[`Hash`]（只能考虑到字段）
/// * 📄OpenNARS `hashCode`：直接使用其（词法上）唯一的「名称」作为依据
///   * ⚠️此处采取更本地化的做法：只散列化与之相关的字段，而无需调用字符串格式化函数
impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
        self.components.hash(state);
    }
}

/// 内建属性
impl Term {
    /// 只读的「标识符」属性
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// 只读的「组分」属性
    pub fn components(&self) -> &TermComponents {
        &self.components
    }

    /// 判断其是否为「占位符」
    /// * 🎯【2024-04-21 01:04:17】在「词法折叠」中首次使用
    pub fn is_placeholder(&self) -> bool {
        self.identifier == PLACEHOLDER
    }

    /// 快捷获取「标识符-组分」二元组
    /// * 🎯用于很多地方的「类型匹配」
    pub fn id_comp(&self) -> (&str, &TermComponents) {
        (&self.identifier, &*self.components)
    }

    /// 快捷获取「标识符-组分」二元组，并提供可变机会
    /// * 🚩【2024-04-21 00:59:20】现在正常返回其两重可变引用
    /// * 📝【2024-04-21 00:58:58】当「标识符」为「静态字串」时，不能对其内部的`&str`属性进行修改
    ///   * 📌使用`&mut &str`会遇到生命周期问题
    ///   * 📌实际上「修改类型」本身亦不常用
    pub fn id_comp_mut(&mut self) -> (&mut str, &mut TermComponents) {
        (&mut self.identifier, &mut *self.components)
    }

    /// 判断「是否包含指定类型的词项」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    pub fn contain_type(&self, identifier: &str) -> bool {
        self.identifier == identifier || self.components.contain_type(identifier)
    }

    /// 判断和另一词项是否「结构匹配」
    /// * 🎯变量替换中的模式匹配
    /// * 🚩类型匹配 & 组分匹配
    /// * ⚠️非递归：不会递归比较「组分是否对应匹配」
    #[inline(always)]
    pub fn structural_match(&self, other: &Self) -> bool {
        self.get_class() == other.get_class() && self.components.structural_match(&other.components)
    }
}

/// 实现[`Display`]
/// * 🎯调试时便于展现内部结构
/// * ⚡性能友好
impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_name())
    }
}

/// 内建属性
impl TermComponents {
    /// 获取「组分」的大小
    /// * ⚠️对于「带索引序列」不包括「索引」
    ///   * 📄对「像」不包括「像占位符」
    pub fn len(&self) -> usize {
        use TermComponents::*;
        match self {
            // 无组分
            Empty | Named(..) => 0,
            // 固定数目
            Unary(..) => 1,
            Binary(..) => 2,
            // 不定数目
            Multi(terms) | MultiIndexed(_, terms) => terms.len(),
        }
    }

    /// 获取「组分是否为空」
    /// * 🎯自clippy提示而设
    pub fn is_empty(&self) -> bool {
        use TermComponents::*;
        match self {
            // 一定空
            Empty | Named(..) => true,
            // 一定非空
            Unary(..) | Binary(..) => false,
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => terms.is_empty(),
        }
    }

    /// 获取指定位置的组分（不一定有）
    /// * ⚠️对于「带索引序列」不受「索引」影响
    ///   * 📄对「像」不受「像占位符」影响
    pub fn get(&self, index: usize) -> Option<&Term> {
        use TermComponents::*;
        match (self, index) {
            // 无组分
            (Empty | Named(..), _) => None,
            // 固定数目 @ 固定索引
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => Some(term),
            // 不定数目
            (Multi(terms) | MultiIndexed(_, terms), _) => terms.get(index),
            // 其它情况⇒无
            _ => None,
        }
    }

    /// 获取指定位置的组分（不检查，直接返回元素）
    /// * ⚠️对于「带索引序列」不受「索引」影响
    ///   * 📄对「像」不受「像占位符」影响
    ///
    /// # Safety
    ///
    /// ⚠️只有在「确保索引不会越界」才不会引发panic和未定义行为（UB）
    pub unsafe fn get_unchecked(&self, index: usize) -> &Term {
        use TermComponents::*;
        match (self, index) {
            // 固定数目
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => term,
            // 不定数目
            (Multi(terms) | MultiIndexed(_, terms), _) => terms.get_unchecked(index),
            // 其它情况⇒panic
            _ => panic!("尝试在非法位置 {index} 获取词项：{self:?}"),
        }
    }

    /// 获取其中「所有元素」的迭代器
    /// * 🚩返回一个迭代器，迭代其中所有「元素」
    /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
    pub fn iter(&self) -> impl Iterator<Item = &Term> {
        use TermComponents::*;
        // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
        let b: Box<dyn Iterator<Item = &Term>> = match self {
            // 一定空
            Empty | Named(..) => Box::new(None.into_iter()),
            // 一定非空
            Unary(term) => Box::new([term].into_iter()),
            Binary(term1, term2) => Box::new([term1, term2].into_iter()),
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter()),
        };
        b
    }

    /// 获取其中「所有元素」的迭代器（可变引用）
    /// * 🚩返回一个迭代器，迭代其中所有「元素」
    /// * 🎯词项的「变量代入」替换
    /// * ⚠️并非「深迭代」：仅迭代自身的下一级词项，不会递归深入
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Term> {
        use TermComponents::*;
        // * 📝必须添加类型注释，以便统一不同类型的`Box`，进而统一「迭代器」类型
        let b: Box<dyn Iterator<Item = &mut Term>> = match self {
            // 一定空
            Empty | Named(..) => Box::new(None.into_iter()),
            // 一定非空
            Unary(term) => Box::new([term].into_iter()),
            Binary(term1, term2) => Box::new([term1, term2].into_iter()),
            // 可能空
            Multi(terms) | MultiIndexed(_, terms) => Box::new(terms.iter_mut()),
        };
        b
    }

    /// 尝试向其中添加元素
    /// * 🚩始终作为其内的「组分」添加，没有「同类⇒组分合并」的逻辑
    /// * 🚩返回「是否添加成功」
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub fn add(&mut self, term: Term) -> bool {
        use TermComponents::*;
        match self {
            // 固定数目的词项⇒必然添加失败
            Empty | Named(..) | Unary(..) | Binary(..) => false,
            // 不定数目⇒添加
            Multi(terms) | MultiIndexed(_, terms) => {
                terms.push(term);
                true
            }
        }
    }

    /// 尝试向其中删除元素
    /// * 🚩始终作为其内的「组分」删除，没有「同类⇒删除其中所有组分」的逻辑
    /// * 🚩返回「是否删除成功」
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub fn remove(&mut self, term: &Term) -> bool {
        use TermComponents::*;
        match self {
            // 固定数目的词项⇒必然添加失败
            Empty | Named(..) | Unary(..) | Binary(..) => false,
            // 不定数目⇒尝试移除
            Multi(terms) | MultiIndexed(_, terms) => match terms.iter().position(|t| t == term) {
                // 找到⇒移除
                Some(index) => {
                    terms.remove(index);
                    true
                }
                // 未找到⇒返回false
                None => false,
            },
        }
    }

    /// 尝试向其中替换元素
    /// * 🚩始终作为其内的「组分」替换
    /// * 🚩返回「是否替换成功」
    /// * ⚠️不涉及「记忆区」有关`make`的「词项缓存机制」
    pub fn replace(&mut self, index: usize, new: Term) -> bool {
        use TermComponents::*;
        match (self, index) {
            // 无组分
            (Empty | Named(..), _) => false,
            // 固定数目 @ 固定索引
            (Unary(term), 0) | (Binary(term, _), 0) | (Binary(_, term), 1) => {
                *term = new;
                true
            }
            // 不定数目 & 长度保证
            (Multi(terms) | MultiIndexed(_, terms), _) if index < terms.len() => {
                terms[index] = new;
                true
            }
            // 其它情况⇒无
            _ => false,
        }
    }

    /// （作为无序不重复集合）重新排序
    /// * 🎯用作「集合中替换元素后，重新排序（并去重）」
    ///   * ⚠️不会在「固定数目词项」中去重
    ///   * 📄NAL-6「变量替换」
    pub fn reorder_unordered(&mut self) {
        use TermComponents::*;
        match self {
            // 空 | 单个
            Empty | Named(..) | Unary(..) => {}
            // 二元 ⇒ 尝试交换 | ⚠️无法去重
            Binary(term1, term2) => {
                if term1 > term2 {
                    std::mem::swap(term1, term2);
                }
            }
            // 不定数目
            Multi(terms) | MultiIndexed(_, terms) => {
                terms.sort_unstable();
                terms.dedup();
            }
        }
    }

    /// 判断「是否包含指定类型的词项」
    /// * 🎯支持「词项」中的方法，递归判断「是否含有变量」
    /// * 🚩【2024-04-21 20:35:23】目前直接基于迭代器
    ///   * 📌牺牲一定性能，加快开发速度
    pub fn contain_type(&self, identifier: &str) -> bool {
        self.iter().any(|term| term.contain_type(identifier))
    }

    /// 判断「结构模式上是否匹配」
    /// * 🚩判断二者在「结构大小」与（可能有的）「结构索引」是否符合
    /// * ⚠️非递归：不会递归比较「组分是否对应匹配」
    /// * 🎯变量替换中的「相同结构之模式替换」
    /// * 📄`variable::find_substitute`
    pub fn structural_match(&self, other: &Self) -> bool {
        use TermComponents::*;
        match (self, other) {
            // 同类型 / 空 | 同类型 / 一元 | 同类型 / 二元
            (Empty | Named(..), Empty | Named(..))
            | (Unary(..), Unary(..))
            | (Binary(..), Binary(..)) => true,
            // 同类型 / 多元
            (Multi(terms1), Multi(terms2)) => terms1.len() == terms2.len(),
            (MultiIndexed(i1, terms1), MultiIndexed(i2, terms2)) => {
                i1 == i2 && terms1.len() == terms2.len()
            }
            // 其它情形（类型相异）
            _ => false,
        }
    }
}
