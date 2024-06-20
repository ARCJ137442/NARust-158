//! 变量处理
//! * 🎯承载所有与「变量」有关的处理
//!
//! ! ⚠️【2024-06-19 23:01:30】此处有关「变量处理」的逻辑尚未稳定：
//!   * 🚧有待在OpenNARS改版中「函数式改造」
// TODO: 【2024-06-20 22:24:43】🚧有待在OpenNARS改版中「函数式改造」
#![allow(unused)]

use crate::language::{CompoundTermRefMut, Term, TermComponents};
use nar_dev_utils::matches_or;
use std::collections::HashMap;

impl CompoundTermRefMut<'_> {
    /// 📄OpenNARS `CompoundTerm.applySubstitute` 方法
    /// * 🚩直接分派给其组分
    /// * 📝OpenNARS中「原子词项」不参与「变量替代」：执行无效果
    ///
    /// # 📄OpenNARS
    ///
    /// Recursively apply a substitute to the current CompoundTerm
    #[inline]
    pub fn apply_substitute(&mut self, substitution: &VarSubstitution) {
        // * 🚩遍历替换内部所有元素
        for inner in self.components() {
            // * 🚩若有「替换方案」⇒替换
            if let Some(substitute_term) = substitution.chain_get(inner) {
                // * ⚠️此处的「被替换词项」可能不是「变量词项」
                // * 📄NAL-6变量引入时会建立「临时共同变量」匿名词项，以替换非变量词项
                // * 🚩一路追溯到「没有再被传递性替换」的词项（最终点）
                let substitute = substitute_term.clone();
                // ! 🚩不使用set_term_when_dealing_variables
                *inner = substitute;
            }
            // * 🚩复合词项⇒递归深入
            if let Some(mut inner_compound) = inner.as_compound_mut() {
                inner_compound.apply_substitute(substitution);
            }
        }
        // * 🚩可交换⇒替换之后重排顺序
        if self.is_commutative() {
            // re-order
            self.reorder_components();
        }
        // * ✅不再需要重新生成名称
    }

    /// 📄OpenNARS `Term.renameVariables` 方法
    /// * 🚩重命名自身变量为一系列「固定编号」
    ///   * 📌整体逻辑：将其中所有不同名称的「变量」编篡到一个字典中，排序后以编号重命名（抹消具体名称）
    ///   * 📝因为这些变量都位于「词项内部」，即「变量作用域全被约束在词项内」，故无需考虑「跨词项编号歧义」的问题
    /// * 📌变量替换的数字索引从`1`开始
    ///   * 📝与变量类型完全无关（from OpenNARS）
    ///     * 📄`(*, $A, #A, ?A)` => `(*, $1, #2, ?3)`
    /// * 🎯用于将「变量」统一命名成固定的整数编号
    /// * ❓目前对此存疑：必要性何在？
    ///   * ~~不一致性：输入`<$A --> $B>`再输入`<$B --> $A>`会被看作是一样的变量~~
    ///   * 📌既然是「变量作用域对整个词项封闭」那**任意名称都没问题**
    ///
    /// # 📄OpenNARS
    ///
    /// @ Term: Blank method to be override in CompoundTerm
    ///
    /// @ CompoundTerm:
    ///   * Rename the variables in the compound, called from Sentence constructors
    ///   * Recursively rename the variables in the compound
    pub fn rename_variables(&mut self) {
        // 创建「变量替换」
        let mut substitution = VarSubstitution::new();
        // 填充「变量映射对」
        // * 🚩从`1`开始
        self.inner().for_each_atom_mut(&mut |atom| {
            // 条件：是变量 & 之前没出现过
            if atom.instanceof_variable() && !substitution.has(atom) {
                // * 🚩替换：类型不变，名称换成「映射大小+1」（唯一的，从1开始）
                substitution.put(atom, Term::make_var_similar(atom, substitution.len() + 1));
            }
        });
        // 应用
        self.apply_substitute(&substitution);
    }
}

/// 📄OpenNARS `Variable.unify` 方法
/// * 🚩总体流程：找「可替换的变量」并（两头都）替换之
/// * 📝❓不对称性：从OpenNARS `findSubstitute`中所见，
///   * `to_be_unified_1`是「包含变量，将要被消元」的那个（提供键），
///   * 而`to_be_unified_2`是「包含常量，将要用于消元」的那个（提供值）
/// * 📌对「在整体中替换部分」有效
///
/// # 📄OpenNARS
///
/// To unify two terms
///
/// @param type            The type of variable that can be substituted
/// @param to_be_unified_1 The first term to be unified
/// @param to_be_unified_2 The second term to be unified
/// @param unified_in_1    The compound containing the first term
/// @param unified_in_2    The compound containing the second term
/// @return Whether the unification is possible
///
/// # 📄案例
///
/// ## 1 from OpenNARS调试 @ 【2024-04-21 21:48:21】
///
/// 传入
///
/// - type: "$"
/// - to_be_unified_1: "<$1 --> B>"
/// - to_be_unified_2: "<C --> B>"
/// - unified_in_1: <<$1 --> A> ==> <$1 --> B>>
/// - unified_in_2: <C --> B>
///
/// 结果
/// - to_be_unified_1: "<$1 --> B>"
/// - to_be_unified_2: "<C --> B>"
/// - unified_in_1: <<C --> A> ==> <C --> B>>
/// - unified_in_2: <C --> B>
///
#[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
pub fn unify(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    unified_in_1: &mut Term,
    unified_in_2: &mut Term,
) -> bool {
    //  寻找
    let (has_substitute, substitution_1, substitution_2) =
        unify_find(var_type, to_be_unified_1, to_be_unified_2);

    // 替换（+更新）
    matches_or! {
        // * 🚩只有在「皆为复合词项」才进行替换
        (unified_in_1.as_compound_mut(), unified_in_2.as_compound_mut()),
        (Some(ref mut compound_1), Some(ref mut compound_2))
            => unify_substitute(compound_1, compound_2, &substitution_1, &substitution_2),
        unreachable!("【2024-06-14 17:03:54】断言：能统一的词项必定是复合词项")
        // TODO: 后续需要看改版中是否能实现「无修改构造替换后词项」
    }

    // 返回「是否替换了变量」
    has_substitute
}

/// `unify`的前半部分
/// * 🎯复用「二词项」和「四词项」，兼容借用规则
/// * 🚩从「将要被统一的词项」中计算出「变量替换映射」
#[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
pub fn unify_find(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
) -> (bool, VarSubstitution, VarSubstitution) {
    let mut substitution_1 = VarSubstitution::new();
    let mut substitution_2 = VarSubstitution::new();
    let has_substitute = find_substitute(
        var_type,
        to_be_unified_1,
        to_be_unified_2,
        &mut substitution_1,
        &mut substitution_2,
    );
    // 返回获取的映射，以及「是否有替换」
    (has_substitute, substitution_1, substitution_2)
}

/// `unify`的前半部分
/// * 🎯复用「二词项」和「四词项」，兼容借用规则
/// * 🚩替换 & 更新
///   * 替换：在「替换所发生在的词项」中根据「变量替换映射」替换词项
///   * 更新：替换后更新词项的「是常量」属性（源自OpenNARS）
pub fn unify_substitute(
    unified_in_1: &mut CompoundTermRefMut,
    unified_in_2: &mut CompoundTermRefMut,
    substitution_1: &VarSubstitution,
    substitution_2: &VarSubstitution,
) {
    // 根据「变量替换映射」在两头相应地替换变量
    // * 🚩若「变量替换映射」为空，本来就不会执行
    unified_in_1.apply_substitute(substitution_1);
    unified_in_2.apply_substitute(substitution_2);
    // 替换后根据「是否已替换」设置词项
    if !substitution_1.is_empty() {
        // 📄 `((CompoundTerm) compound1).renameVariables();`
        // 📄 `setConstant(true);` @ `CompoundTerm`
        // unified_in_1.is_constant = true;
        todo!("TODO: 尚需在OpenNARS中假定「找到了变量替换映射，就一定是复合词项」")
    }
    if !substitution_2.is_empty() {
        // 📄 `((CompoundTerm) compound2).renameVariables();`
        // 📄 `setConstant(true);` @ `CompoundTerm`
        // unified_in_2.is_constant = true;
        todo!("TODO: 尚需在OpenNARS中假定「找到了变量替换映射，就一定是复合词项」")
    }
}

/// 📄OpenNARS `Variable.findSubstitute` 方法
/// * 💫【2024-04-21 21:40:45】目前尚未能完全理解此处的逻辑
/// * 📝【2024-04-21 21:50:42】递归查找一个「同位替代」的「变量→词项」映射
/// * 🚧缺少注释：逻辑基本照抄OpenNARS的代码
///
/// # 📄OpenNARS
///
/// To recursively find a substitution that can unify two Terms without changing them
///
/// @param type            The type of variable that can be substituted
/// @param to_be_unified_1 The first term to be unified
/// @param to_be_unified_2 The second term to be unified
/// @param substitution_1  The substitution for term1 formed so far
/// @param substitution_2  The substitution for term2 formed so far
/// @return Whether the unification is possible
///
/// # 📄案例
///
/// ## 1 from OpenNARS调试 @ 【2024-04-21 21:48:21】
///
/// 传入
///
/// - type: "$"
/// - to_be_unified_1: "<$1 --> B>"
/// - to_be_unified_2: "<C --> B>"
/// - substitution_1: HashMap{}
/// - substitution_2: HashMap{}
///
/// 结果
///
/// - 返回值 = true
/// - substitution_1: HashMap{ Term"$1" => Term"C" }
/// - substitution_2: HashMap{}
///
/// ## 2 from OpenNARS调试 @ 【2024-04-21 22:05:46】
///
/// 传入
///
/// - type: "$"
/// - to_be_unified_1: "<<A --> $1> ==> <B --> $1>>"
/// - to_be_unified_2: "<B --> C>"
/// - substitution_1: HashMap{}
/// - substitution_2: HashMap{}
///
/// 结果
///
/// - 返回值 = true
/// - substitution_1: HashMap{ Term"$1" => Term"C" }
/// - substitution_2: HashMap{}
#[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
pub fn find_substitute(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    substitution_1: &mut VarSubstitution,
    substitution_2: &mut VarSubstitution,
) -> bool {
    //==== 内用函数 ====//

    /// 特殊的「共有变量」标识符
    /// * 📄迁移自OpenNARS
    const COMMON_VARIABLE: &str = "COMMON_VARIABLE";

    /// 📄OpenNARS `Variable.makeCommonVariable` 函数
    /// * 🎯用于「变量统一」方法
    fn make_common_variable(v1: &Term, v2: &Term) -> Term {
        Term::new(
            COMMON_VARIABLE,
            TermComponents::Word(v1.name() + &v2.name()),
        )
    }

    /// 📄OpenNARS `Variable.isCommonVariable` 函数
    fn is_common_variable(v: &Term) -> bool {
        v.identifier() == COMMON_VARIABLE
    }

    //==== 正式开始函数体 ====//
    // 📄 `if ((term1 instanceof Variable) && (((Variable) term1).getType() == type)) {`
    if to_be_unified_1.get_variable_type() == var_type {
        match substitution_1.get(to_be_unified_1).cloned() {
            // already mapped
            Some(new_term) => {
                // 📄 `return findSubstitute(type, t, term2, map1, map2);`
                // 在新替换的变量中递归深入
                find_substitute(
                    var_type,
                    &new_term, // ! 必须复制：否则会存留不可变引用
                    to_be_unified_2,
                    substitution_1,
                    substitution_2,
                )
            }
            // not mapped yet
            None => {
                if to_be_unified_2.get_variable_type() == var_type {
                    let common_var = make_common_variable(to_be_unified_1, to_be_unified_2);
                    substitution_1.put(to_be_unified_1, common_var.clone()); // unify
                    substitution_2.put(to_be_unified_2, common_var); // unify
                } else {
                    substitution_1.put(to_be_unified_1, to_be_unified_2.clone()); // elimination
                    if is_common_variable(to_be_unified_1) {
                        substitution_2.put(to_be_unified_1, to_be_unified_2.clone());
                    }
                }
                true
            }
        }
    } else if to_be_unified_2.get_variable_type() == var_type {
        // 📄 `else if ((term2 instanceof Variable) && (((Variable) term2).getType() == type)) {`
        // 📄 `t = map2.get(var2); if (t != null) { .. }`
        match substitution_2.get(to_be_unified_2).cloned() {
            // already mapped
            Some(new_term) => {
                find_substitute(
                    var_type,
                    to_be_unified_1,
                    &new_term, // ! 必须复制：否则会存留不可变引用
                    substitution_1,
                    substitution_2,
                )
            }
            // not mapped yet
            None => {
                /*
                 * 📝【2024-04-22 00:13:19】发生在如下场景：
                 * <(&&, <A-->C>, <B-->$2>) ==> <C-->$2>>.
                 * <(&&, <A-->$1>, <B-->D>) ==> <$1-->D>>.
                 * <(&&, <A-->C>, <B-->D>) ==> <C-->D>>?
                 *
                 * 系列调用：
                 * * `$` `A` `$1`
                 * * `$` `D` `$1`
                 * * `$` `<C --> D>` `<$1 --> D>`
                 * * `$` `<C --> D>` `<C --> $1>`
                 *
                 * 📌要点：可能两边各有「需要被替换」的地方
                 */
                substitution_2.put(to_be_unified_2, to_be_unified_1.clone()); // elimination
                if is_common_variable(to_be_unified_2) {
                    substitution_1.put(to_be_unified_2, to_be_unified_1.clone());
                }
                true
            }
        }
    } else if to_be_unified_1.instanceof_compound() {
        // 必须结构匹配
        // 📄 `if (cTerm1.size() != ...... return false; }`
        if to_be_unified_1.structural_match(to_be_unified_2) {
            // 📄 `else if ((term1 instanceof CompoundTerm) && term1.getClass().equals(term2.getClass())) {`
            // ? ❓为何要打乱无序词项——集合词项的替换过于复杂，只能用「随机打乱」间接尝试所有组合
            // 📄 `if (cTerm1.isCommutative()) { Collections.shuffle(list, Memory.randomNumber); }`
            // TODO: 🏗️有关无序复合词项的「变量统一」需要进一步处理——不希望采用「随机打乱」的方案，可能要逐个枚举匹配
            // ! 边缘情况：`<(*, $1, $2) --> [$1, $2]>` => `<(*, A, A) --> [A]>`
            // ! 边缘情况：   `<<A --> [$1, $2]> ==> <A --> (*, $1, $2)>>`
            // ! 　　　　　+  `<A --> [B, C]>` |- `<A --> (*, B, C)>`✅
            // ! 　　　　　+  `<A --> [B]>` |- `<A --> (*, B, B)>`❌
            // ! 🚩【2024-04-22 09:43:26】此处暂且不打乱无序词项：疑点重重
            // 对位遍历
            // for (t1, t2) in to_be_unified_1
            //     .get_components()
            //     .zip(to_be_unified_2.get_components())
            // {
            //     if !find_substitute(var_type, t1, t2, substitution_1, substitution_2) {
            //         return false;
            //     }
            // }
            // * 🚩【2024-04-22 09:45:55】采用接近等价的纯迭代器方案，可以直接返回
            if let (
                Some(CompoundTermRef {
                    components: components1,
                    ..
                }),
                Some(CompoundTermRef {
                    components: components2,
                    ..
                }),
            ) = (to_be_unified_1.as_compound(), to_be_unified_2.as_compound())
            {
                components1.iter().zip(components2.iter()).all(|(t1, t2)| {
                    find_substitute(var_type, t1, t2, substitution_1, substitution_2)
                })
            } else {
                false
            }
        } else {
            // 复合词项结构不匹配，一定不能替代
            false
        }
    } else {
        // for atomic constant terms
        to_be_unified_1 == to_be_unified_2
    }
}

/// 📄OpenNARS `Variable.hasSubstitute` 方法
/// * 🚩判断「是否有可能被替换」
///   * ⚠️反常情况：即便是「没有变量需要替换」，只要「模式有所匹配」就能发生替换
///
/// # 📄OpenNARS
///
/// Check if two terms can be unified
///
///  @param type  The type of variable that can be substituted
///  @param term1 The first term to be unified
///  @param term2 The second term to be unified
///  @return Whether there is a substitution
#[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
pub fn has_substitute(var_type: &str, to_be_unified_1: &Term, to_be_unified_2: &Term) -> bool {
    // 📄 `return findSubstitute(type, term1, term2, new HashMap<Term, Term>(), new HashMap<Term, Term>());`
    find_substitute(
        var_type,
        to_be_unified_1,
        to_be_unified_2,
        // 创建一个临时的「变量替换映射」
        &mut VarSubstitution::new(),
        &mut VarSubstitution::new(),
    )
}

impl TermComponents {
    #[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
    /// 📄OpenNARS `CompoundTerm.applySubstitute` 方法
    pub fn apply_substitute(&mut self, substitution: &VarSubstitution) {
        // 遍历其中所有地方的可变引用
        for term in self.iter_mut() {
            // 寻找其「是否有替代」
            match substitution.get(term) {
                // 有替代⇒直接赋值
                Some(new_term) => *term = new_term.clone(),
                // 没替代⇒继续递归替代
                None => term.apply_substitute(substitution),
            }
        }
    }
}

/// 用于表示「变量替换」的字典
/// * 🎯NAL-6中的「变量替换」「变量代入」
#[derive(Debug, Default, Clone)]
#[doc(alias = "VariableSubstitution")]
pub struct VarSubstitution {
    map: HashMap<Term, Term>,
}

impl VarSubstitution {
    /// 构造函数
    pub fn new() -> Self {
        Self::default()
    }

    /// 从其它构造出「散列映射」的地方构造
    pub fn from(map: impl Into<HashMap<Term, Term>>) -> Self {
        Self { map: map.into() }
    }

    /// 从其它构造出「散列映射」的地方构造
    pub fn from_pairs(pairs: impl IntoIterator<Item = (Term, Term)>) -> Self {
        Self {
            map: HashMap::from_iter(pairs),
        }
    }

    /// 尝试获取「替代项」
    /// * 🎯变量替换
    pub fn get(&self, key: &Term) -> Option<&Term> {
        self.map.get(key)
    }

    /// 链式获取「变量替换」最终点
    /// * 🚩一路查找到头
    /// * 📄{A -> B, B -> C}, A => Some(C)
    /// * 📄{A -> B, B -> C}, B => Some(C)
    /// * 📄{A -> B, B -> C}, C => None
    pub fn chain_get(&self, key: &Term) -> Option<&Term> {
        // * ⚠️此时应该传入非空值
        // * 🚩从「起始点」开始查找
        let mut end_point = self.get(key)?;
        // * 🚩非空⇒一直溯源
        loop {
            match self.get(end_point) {
                Some(next_point) => {
                    debug_assert!(end_point != next_point, "不应有循环替换之情况！");
                    end_point = next_point
                }
                None => break Some(end_point),
            }
        }
    }

    /// 尝试判断「是否有键」
    /// * 🎯变量重命名
    pub fn has(&self, key: &Term) -> bool {
        self.map.contains_key(key)
    }

    /// 获取「可替换的变量个数」
    /// * 🚩映射的大小
    /// * 🎯变量重命名
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 判断「是否为空」
    /// * 🎯变量替换后检查「是否已替换」
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 设置「替代项」
    /// * 🎯寻找可替换变量，并返回结果
    /// * 🚩只在没有键时复制`key`，并且总是覆盖`value`值
    pub fn put(&mut self, key: &Term, value: Term) {
        match self.map.get_mut(key) {
            Some(old_value) => *old_value = value,
            None => {
                self.map.insert(key.clone(), value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_term as term;
    use crate::util::AResult;
    use nar_dev_utils::macro_once;

    #[cfg(TODO_20240614164500_有待迁移)] // TODO: 有待后续完成迁移后改写
    /// 测试/变量替换
    #[test]
    fn apply_substitute() -> AResult {
        let substitution = VarSubstitution::from_pairs([
            (term!("var_word"), term!("word")),
            (term!("$1"), term!("1")),
        ]);
        macro_once! {
            macro apply_substitute (
                $(
                    $term_str:expr, $substitution:expr
                    => $substituted_str:expr
                )*
            ) {
                $(
                    let mut term = term!($term_str);
                    term.apply_substitute(&$substitution);
                    assert_eq!(term, term!($substituted_str));
                )*
            }
            "<A --> var_word>", substitution => "<A --> word>"
            "<<$1 --> A> ==> <B --> $1>>", substitution => "<<1 --> A> ==> <B --> 1>>"
        }
        ok!()
    }

    // TODO: 后续重写「变量重命名」机制
    #[cfg(弃用_20240614000709_需要重写)]
    /// 测试 / unify | unify_two
    #[test]
    fn unify() -> AResult {
        use crate::language::variable::unify_two;
        macro_once! {
            macro unify(
                $(
                    $term_str1:expr, $term_str2:expr
                    => $var_type:expr =>
                    $substituted_str1:expr, $substituted_str2:expr
                )*
            ) {
                $(
                    let mut term1 = term!($term_str1);
                    let mut term2 = term!($term_str2);
                    let var_type = $var_type;
                    print!("unify: {}, {} =={var_type}=> ", term1.format_name(), term2.format_name());
                    unify_two($var_type, &mut term1, &mut term2);
                    let expected_1 = term!($substituted_str1);
                    let expected_2 = term!($substituted_str2);
                    println!("{}, {}", term1.format_name(), term2.format_name());
                    assert_eq!(term1, expected_1);
                    assert_eq!(term2, expected_2);
                )*
            }
            // ! 变量替换只会发生在复合词项之中：原子词项不会因此改变自身 //
            "$1", "A" => "$" => "$1", "A"

            // 各个位置、各个角度（双向）的替换 //
            // 单侧偏替换
            "<$1 --> B>", "<A --> B>" => "$" => "<A --> B>", "<A --> B>"
            "<A --> $1>", "<A --> B>" => "$" => "<A --> B>", "<A --> B>"
            "<A --> B>", "<$1 --> B>" => "$" => "<A --> B>", "<A --> B>"
            "<A --> B>", "<A --> $1>" => "$" => "<A --> B>", "<A --> B>"
            // 双侧偏替换
            "<$a --> B>", "<A --> $b>" => "$" => "<A --> B>", "<A --> B>"
            // 单侧全替换
            "<A --> B>", "<$a --> $b>" => "$" => "<A --> B>", "<A --> B>"

            // 三种变量正常运行 & 一元复合词项 //
            "(--, $1)", "(--, 1)" => "$" => "(--, 1)", "(--, 1)"
            "(--, #1)", "(--, 1)" => "#" => "(--, 1)", "(--, 1)"
            "(--, ?1)", "(--, 1)" => "?" => "(--, 1)", "(--, 1)"
            // ! ⚠️【2024-04-22 12:32:47】以下示例失效：第二个例子中，OpenNARS在「第一个失配」后，就无心再匹配第二个了
            // "(*, $i, #d, ?q)", "(*, I, D, Q)" => "$" => "(*, I, #d, ?q)", "(*, I, D, Q)"
            // "(*, $i, #d, ?q)", "(*, I, D, Q)" => "#" => "(*, $i, D, ?q)", "(*, I, D, Q)"
            // "(*, $i, #d, ?q)", "(*, I, D, Q)" => "?" => "(*, $i, #d, Q)", "(*, I, D, Q)"

            // 多元复合词项（有序）：按顺序匹配 //
            "(*, $c, $b, $a)", "(*, (--, C), <B1 --> B2>, A)" => "$" => "(*, (--, C), <B1 --> B2>, A)", "(*, (--, C), <B1 --> B2>, A)"

            // 无序词项 | ⚠️【2024-04-22 12:38:38】对于无序词项的「模式匹配」需要进一步商酌 //
            "{$c}", "{中心点}" => "$" => "{中心点}", "{中心点}" // 平凡情况
            "[$c]", "[中心点]" => "$" => "[中心点]", "[中心点]" // 平凡情况
            // "<$a <-> Bb>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 无需交换顺序，但会被自动排序导致「顺序不一致」
            // "<Aa <-> $b>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 无需交换顺序，但会被自动排序导致「顺序不一致」
            // "<$a <-> $b>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 无需交换顺序，但会被自动排序导致「顺序不一致」
            // "<Bb <-> $a>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 顺序不一致
            // "<$b <-> Aa>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 顺序不一致
            // "<$b <-> $a>", "<Aa <-> Bb>" => "$" => "<Aa <-> Bb>", "<Aa <-> Bb>" // 顺序不一致
            // 平凡情况
            // "{$1,2,3}", "{0, 2, 3}" => "$" => "{0, 2, 3}", "{0, 2, 3}"
            // "{1,$2,3}", "{1, 0, 3}" => "$" => "{1, 0, 3}", "{1, 0, 3}"
            // "{1,2,$3}", "{1, 2, 0}" => "$" => "{1, 2, 0}", "{1, 2, 0}"
            // 无序集合×复合
            // "{1, (*, X), (*, $x)}", "{1, (*, Y), (*, X)}" => "$" => "{1, (*, Y), (*, X)}", "{1, (*, Y), (*, X)}"
        }
        ok!()
    }

    // TODO: 后续重写「变量重命名」机制
    #[cfg(弃用_20240614000709_需要重写)]
    #[test]
    fn rename_variables() -> AResult {
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期词项字符串
            macro rename_variables($($term:literal => $expected:expr )*) {
                $(
                    // 解析构造词项
                    let mut term = term!($term);
                    print!("{term}");
                    // 重命名变量
                    term.rename_variables();
                    println!("=> {term}");
                    // 比对
                    // dbg!(&term);
                    // assert_eq!(term, term!($expected));
                )*
            }
            // 简单情况（一层） //
            // 占位符
            "_" => "_"
            // 原子词项不变
            "A" => "A"
            "$A" => "$A"
            "#A" => "#A"
            "?A" => "?A"
            // 复合词项
            "{$A, $B}" => "{$1, $2}"
            "[$A, $B]" => "[$1, $2]"
            "(&, $A, $B)" => "(&, $1, $2)"
            "(|, $A, $B)" => "(|, $1, $2)"
            "(-, $A, $B)" => "(-, $1, $2)"
            "(~, $A, $B)" => "(~, $1, $2)"
            "(*, $A, $B)" => "(*, $1, $2)"
            r"(/, $R, _)" => r"(/, $1, _)"
            r"(\, $R, _)" => r"(\, $1, _)"
            r"(/, $R, _, $A)" => r"(/, $1, _, $2)"
            r"(\, $R, _, $A)" => r"(\, $1, _, $2)"
            r"(&&, $A, $B)" => r"(&&, $1, $2)"
            r"(||, $A, $B)" => r"(||, $1, $2)"
            r"(--, $A)" => r"(--, $1)"
            // 陈述
            "<$A --> $B>" => "<$1 --> $2>"
            "<$A <-> $B>" => "<$1 <-> $2>"
            "<$A ==> $B>" => "<$1 ==> $2>"
            "<$A <=> $B>" => "<$1 <=> $2>"
            // 复杂情况 //
            // 不同变量类型，数值不会重复
            "(*, $A, #A, ?A)" => "(*, $1, #2, ?3)"
            // 复合词项：递归深入
            "(&&, A, $B, [C, #D])" => "(&&, A, $1, [C, #2])"
            "<(--, (--, (--, (--, (--, (--, (--, (--, A)))))))) --> (/, (-, ?B, C), _, (/, (/, (/, (/, (/, #D, _), _), _), _), _))>" => "<(--, (--, (--, (--, (--, (--, (--, (--, A)))))))) --> (/, (-, ?1, C), _, (/, (/, (/, (/, (/, #2, _), _), _), _), _))>"
            "<<A --> $B> ==> <#C --> D>>" => "<<A --> $1> ==> <#2 --> D>>"
            "<<A --> #B> ==> <$B --> D>>" => "<<A --> #1> ==> <$2 --> D>>"
            // 相同变量，数值相同
            "<<A --> $B> ==> <$B --> D>>" => "<<A --> $1> ==> <$1 --> D>>"
            "(*, $A, $A, $A)" => "(*, $1, $1, $1)"
            "(*, (*, $A, $A, $A), (*, $A, $A, $A), (*, $A, $A, $A))" => "(*, (*, $1, $1, $1), (*, $1, $1, $1), (*, $1, $1, $1))"
        }
        ok!()
    }
}
