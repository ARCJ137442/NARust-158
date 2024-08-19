//! 变量处理
//! * 🎯承载所有与「变量」有关的处理
//!
//! ! ⚠️【2024-06-19 23:01:30】此处有关「变量处理」的逻辑尚未稳定：
//!   * 🚧有待在OpenNARS改版中「函数式改造」

use crate::{
    language::{CompoundTermRef, CompoundTermRefMut, Term},
    symbols::*,
};
use nar_dev_utils::void;
use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
use std::{collections::HashMap, ops::BitAnd};

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
                    debug_assert!(
                        end_point != key,
                        "不应有循环替换之情况！{key} @ {self:?}"
                    );
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
            // * 🚩有键⇒覆盖
            Some(old_value) => *old_value = value,
            // * 🚩无键⇒插入
            None => void(self.map.insert(key.clone(), value)),
        }
    }

    /// 删除映射中的「恒等替换」
    /// * 📄`$1 => $1`
    pub fn reduce_identities(&mut self) {
        // * 🚩直接调用内置方法
        self.map.retain(|k, v| k != v);
    }
}

impl CompoundTermRefMut<'_> {
    /// 📄OpenNARS `CompoundTerm.applySubstitute` 方法
    /// * 🚩直接分派给其组分
    /// * 📝OpenNARS中「原子词项」不参与「变量替代」：执行无效果
    ///
    /// # 📄OpenNARS
    ///
    /// Recursively apply a substitute to the current CompoundTerm
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
        // * 📝【2024-08-08 13:03:24】所谓「共同变量」总会有所谓「泄漏」的问题
        //   * 💡关键在于「是否最终能被当作『普通变量』对待」
        //   * 🚩方案：将其就视作「普通变量」，判别方式就是「是否在词项本身域外」
        // // 检查是否会有「共同变量泄漏」问题
        // if cfg!(debug_assertions) {
        //     self.for_each_atom(&mut |atom| {
        //         debug_assert!(
        //             !is_common_variable(atom) || substitution.chain_get(atom).is_some(),
        //             "common variable {atom} leaked!\nsubstitution = {substitution:?}"
        //         )
        //     });
        // }
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
        substitution.reduce_identities();
        // 应用
        self.apply_substitute(&substitution);
    }
}

/// `unify`的前半部分
/// * 🎯复用「二词项」和「四词项」，兼容借用规则
/// * 🚩从「将要被统一的词项」中计算出「变量替换映射」
fn unify_find(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> Unification {
    let mut unify_map_1 = VarSubstitution::new();
    let mut unify_map_2 = VarSubstitution::new();
    let has_unification = find_unification(
        var_type,
        to_be_unified_1,
        to_be_unified_2,
        &mut unify_map_1,
        &mut unify_map_2,
        shuffle_rng_seed,
    );
    // 返回获取的映射，以及「是否有替换」
    Unification {
        has_unification,
        unify_map_1,
        unify_map_2,
    }
}

/// 【对外接口】统一独立变量
pub fn unify_find_i(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> Unification {
    unify_find(
        VAR_INDEPENDENT,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

/// 【对外接口】统一非独变量
pub fn unify_find_d(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> Unification {
    unify_find(
        VAR_DEPENDENT,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

/// 【对外接口】统一查询变量
pub fn unify_find_q(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> Unification {
    unify_find(
        VAR_QUERY,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

/// 多值输出：寻找「归一替换」的中间结果
/// * 🎯使用类似`unify_find(t1, t2).apply_to(c1, c2)`完成「可变性隔离」
#[derive(Debug, Clone)]
pub struct Unification {
    /// 是否能归一
    pub has_unification: bool,
    /// 如若归一，归一要换掉的变量映射 @ 词项1
    pub unify_map_1: VarSubstitution,
    /// 如若归一，归一要换掉的变量映射 @ 词项2
    pub unify_map_2: VarSubstitution,
}

impl Unification {
    /// 重定向到[`unify_apply`]
    /// * 🚩返回「是否可归一化」
    /// * 🚩【2024-07-09 21:48:43】目前作为一个实用的「链式应用方法」用以替代公开的`unifyApply`
    #[inline]
    pub fn apply_to(&self, parent1: CompoundTermRefMut, parent2: CompoundTermRefMut) -> bool {
        unify_apply(parent1, parent2, self)
    }

    /// 同[`Self::apply_to`]，但允许应用在任何词项中
    /// * 🚩一律返回「是否已归一化」
    ///   * ⚠️对「单个复合词项」仍可能应用归一化失败：与「应用到哪儿」无关
    pub fn apply_to_term(&self, parent1: &mut Term, parent2: &mut Term) -> bool {
        // * 🚩只有俩词项是复合词项时，才进行应用
        match [parent1.as_compound_mut(), parent2.as_compound_mut()] {
            [Some(parent1), Some(parent2)] => self.apply_to(parent1, parent2),
            _ => self.has_unification,
        }
    }
}

/// 使用「统一结果」统一两个复合词项
/// * ⚠️会修改原有的复合词项
///
/// @param parent1 [&m] 要被修改的复合词项1
/// @param parent2 [&m] 要被修改的复合词项2
/// @param result  [] 上一个「寻找归一映射」的结果
fn unify_apply(
    unified_in_1: CompoundTermRefMut,
    unified_in_2: CompoundTermRefMut,
    unification: &Unification,
) -> bool {
    let Unification {
        has_unification,
        unify_map_1,
        unify_map_2,
    } = unification;
    // 根据「变量替换映射」在两头相应地替换变量
    apply_unify_one(unified_in_1, unify_map_1);
    apply_unify_one(unified_in_2, unify_map_2);
    *has_unification
}

/// 得出「替代结果」后，将映射表应用到词项上
fn apply_unify_one(mut unified_in: CompoundTermRefMut, substitution: &VarSubstitution) {
    // * 🚩映射表非空⇒替换
    if substitution.is_empty() {
        return;
    }
    // * 🚩应用 & 重命名
    unified_in.apply_substitute(substitution);
    // 替换后设置词项
    // 📄 `((CompoundTerm) compound1).renameVariables();`
    // 📄 `setConstant(true);` @ `CompoundTerm`
    // unified_in_1.is_constant = true;
    unified_in.rename_variables();
}

/// 🆕将上述方法放在映射表的方法上
impl VarSubstitution {
    /// 将映射表的替换模式应用到「复合词项可变引用」上
    /// * 🎯用于「只需单个替换」的情况
    ///   * 📄首先出自「条件演绎/归纳」
    pub fn apply_to(&self, to: CompoundTermRefMut) {
        apply_unify_one(to, self)
    }

    /// 尝试将映射表的替换模式应用到任意词项上
    /// * 🎯用于「先应用，再判断词项类型」的情况
    #[inline]
    pub fn apply_to_term(&self, to: &mut Term) {
        if let Some(to) = to.as_compound_mut() {
            // 传入（因此可内联）
            self.apply_to(to);
        }
    }
}

/// 多值输出：寻找「归一替换」的中间结果
/// ! ❌【2024-07-09 21:14:17】暂且不复刻`unifyApplied`：自成体系但不完整，需要结合`applyUnifyToNew`等「函数式方法」
pub type AppliedCompounds = [Term; 2];

/// 判断两个复合词项是否「容器相同」
/// * 🚩只判断有关「怎么包含词项」的信息，不判断具体内容
fn is_same_kind_compound(t1: CompoundTermRef, t2: CompoundTermRef) -> bool {
    // * 🚩判断尺寸
    if t1.size() != t2.size() {
        return false;
    }
    // * 🚩判断「像」的关系位置（占位符位置）
    if (t1.instanceof_image() && t2.instanceof_image())
        && t1.get_placeholder_index() != t2.get_placeholder_index()
    {
        // 均为像，但占位符位置不同⇒否决
        return false;
    }
    // * 🚩验证通过
    true
}

/// 📄OpenNARS `Variable.findSubstitute` 方法
/// * 💫【2024-04-21 21:40:45】目前尚未能完全理解此处的逻辑
/// * 📝【2024-04-21 21:50:42】递归查找一个「同位替代」的「变量→词项」映射
/// * ⚠️【2024-07-10 14:40:06】目前对「可交换词项」沿用OpenNARS的「随机打乱」方案
///   * ✅能保证「推理器相同，随机运行的结果不因系统时间而变」
///   * 💫因借用问题，需要每次使用时引入一个「随机种子」作为随机因子
///
/// # 📄OpenNARS
///
/// To recursively find a substitution that can unify two Terms without changing them
///
/// @param type            The type of variable that can be substituted
/// @param to_be_unified_1 The first term to be unified
/// @param to_be_unified_2 The second term to be unified
/// @param map_1  The substitution for term1 formed so far
/// @param map_2  The substitution for term2 formed so far
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
/// - map_1: HashMap{}
/// - map_2: HashMap{}
///
/// 结果
///
/// - 返回值 = true
/// - map_1: HashMap{ Term"$1" => Term"C" }
/// - map_2: HashMap{}
///
/// ## 2 from OpenNARS调试 @ 【2024-04-21 22:05:46】
///
/// 传入
///
/// - type: "$"
/// - to_be_unified_1: "<<A --> $1> ==> <B --> $1>>"
/// - to_be_unified_2: "<B --> C>"
/// - map_1: HashMap{}
/// - map_2: HashMap{}
///
/// 结果
///
/// - 返回值 = true
/// - map_1: HashMap{ Term"$1" => Term"C" }
/// - map_2: HashMap{}
fn find_unification(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    map_1: &mut VarSubstitution,
    map_2: &mut VarSubstitution,
    shuffle_rng_seed: u64,
) -> bool {
    struct UnificationStatus<'s> {
        /// 统一的变量类型
        var_type: &'s str,
        /// 需要统一的俩词项中，最大的变量id
        max_var_id: usize,
        // /// 根部词项1
        // root_1: &'s Term,
        // /// 根部词项2
        // root_2: &'s Term,
    }

    // 构造状态：原先用闭包能捕获的所有【不变】常量
    let status = UnificationStatus {
        var_type,
        max_var_id: Term::maximum_variable_id_multi([to_be_unified_1, to_be_unified_2]),
        // root_1: to_be_unified_1,
        // root_2: to_be_unified_2,
    };

    impl UnificationStatus<'_> {
        /// 是【确定需要归一化】的变量
        /// * 📄临时的「共用变量」
        /// * 📄满足指定标识符的变量词项
        /// * 🚩【2024-07-09 22:46:21】因为要捕获「变量类型」故需使用闭包
        /// * 📝【2024-07-09 22:47:34】OpenNARS中似乎只在 `to_be_unified_1` 中出现「共用变量」
        fn as_correct_var<'t>(&self, t: &'t Term) -> Option<(&'t Term, usize)> {
            t.as_variable() // 首先是个「变量」词项
                .filter(|_| t.get_variable_type() == self.var_type) // 类型必须是指定类型
                .map(|id| (t, id)) // 需要附带词项引用，以便后续拷贝
        }

        /// 📄OpenNARS `Variable.isCommonVariable` 函数
        /// * 🚩【2024-08-08 13:22:09】现在不再使用特别的标识符，而是与「变量词项」一视同仁——只判断是否为「根部之外」的变量
        ///   * id小于原先的「最大id」 ⇒ 一定是「新创的变量」 ⇒ 一定是「共同变量」
        #[inline]
        fn is_common_variable(&self, v: &Term) -> bool {
            v.as_variable().is_some_and(|id| id > self.max_var_id)
        }

        /// 制作一个由id1 id2共同决定的、在词项自身变量范围之外的id
        /// * 📌假定：自身的「最大变量id」大于0，即 `max_var_id > 0`
        ///   * 💭若根部词项没变量，就不会执行「创建共同变量」的操作
        /// * 📝原理 & 证明
        ///   * ℹ️前提：`id1 ∈ [0, max_var_id]`、`id2 ∈ [0, max_var_id]`
        ///   * 📍推论：`(max_var_id + 1) * (1 + id1) ≥ max_var_id + 1 > max_var_id`
        ///     * ✅满足「在词项自身变量范围之外」
        ///   * 📍推论：`(max_var_id + 1) * (1 + id1) + id2 ≤ max_id_1 = (max_var_id + 1) * (1 + id1) + max_var_id]`
        ///     *  `(max_var_id + 1) * (1 + (id1 + 1)) + id2 ≥ max_id_next = (max_var_id + 1) * (1 + (id1 + 1))`
        ///     *  `max_id_1 = (max_var_id + 1) * (1 + id1) + max_var_id < (max_var_id + 1) * (1 + id1) + (max_var_id + 1) = (max_var_id + 1) * (1 + (id1 + 1)) = max_id_next`
        fn common_var_id_from(&self, id1: usize, id2: usize) -> usize {
            (self.max_var_id + 1) * (1 + id1) + id2
        }

        /// 📄OpenNARS `Variable.makeCommonVariable` 函数
        /// * 📌制作临时的「共用变量」词项
        /// * 🎯用于「变量统一」方法
        /// * 🚩【2024-08-08 13:43:24】现在创建一个新的「域外变量」代替
        #[inline]
        fn make_common_variable(&self, id1: usize, id2: usize) -> Term {
            Term::new_var(self.var_type, self.common_var_id_from(id1, id2))
        }
    }

    /// 递归用子函数
    fn find_unification_sub(
        status: &UnificationStatus,
        [to_be_unified_1, to_be_unified_2]: [&Term; 2],
        [map_1, map_2]: [&mut VarSubstitution; 2],
        shuffle_rng_seed: u64, // ! 在递归传入时刷新
    ) -> bool {
        let is_same_type = to_be_unified_1.is_same_type(to_be_unified_2);
        match [
            status.as_correct_var(to_be_unified_1),
            status.as_correct_var(to_be_unified_2),
        ] {
            // * 🚩[$1 x ?] 对应位置是变量
            // * 🚩[$1 x $2] 若同为变量⇒统一二者（制作一个「共同变量」）
            [Some((var_1, id1)), Some((var_2, id2))] => {
                // * 🚩已有替换⇒直接使用已有替换（看子项有无替换） | 递归深入
                // already mapped
                if let Some(ref mapped) = map_1.get(var_1).cloned() {
                    return find_unification_sub(
                        status,
                        [mapped, to_be_unified_2],
                        [map_1, map_2],
                        shuffle_rng_seed,
                    );
                }
                // not mapped yet
                // * 🚩生成一个外界输入中不可能的变量词项作为「匿名变量」
                let common_var = status.make_common_variable(id1, id2);
                // * 🚩建立映射：var1 -> commonVar @ term1
                // * 🚩建立映射：var2 -> commonVar @ term2
                map_1.put(var_1, common_var.clone()); // unify
                map_2.put(var_2, common_var); // unify
                true
            }
            // * 🚩[$1 x _2] 若并非变量⇒尝试消元划归
            // * 📝此处意味「两个变量合并成一个变量」 | 后续「重命名变量」会将其消去
            [Some((var_1, _)), None] => {
                // * 🚩已有替换⇒直接使用已有替换（看子项有无替换） | 递归深入
                // already mapped
                if let Some(ref mapped) = map_1.get(var_1).cloned() {
                    return find_unification_sub(
                        status,
                        [mapped, to_be_unified_2],
                        [map_1, map_2],
                        shuffle_rng_seed,
                    );
                }
                // * 🚩建立映射：var1 -> term2 @ term1
                // elimination
                map_1.put(var_1, to_be_unified_2.clone());
                // * 🚩尝试消除「共同变量」
                if status.is_common_variable(var_1) {
                    // * 🚩建立映射：var1 -> term2 @ term2
                    map_2.put(var_1, to_be_unified_2.clone());
                }
                true
            }
            // * 🚩[? x $2] 对应位置是变量
            [None, Some((var_2, _))] => {
                // * 🚩已有替换⇒直接使用已有替换（看子项有无替换） | 递归深入
                // already mapped
                if let Some(ref mapped) = map_2.get(var_2).cloned() {
                    return find_unification_sub(
                        status,
                        [to_be_unified_1, mapped],
                        [map_1, map_2],
                        shuffle_rng_seed,
                    );
                }
                // not mapped yet
                // * 🚩[_1 x $2] 若非变量⇒尝试消元划归
                /*
                 * 📝【2024-04-22 00:13:19】发生在如下场景：
                 * <(&&, <A-->C>, <B-->$2>) ==> <C-->$2>>.
                 * <(&&, <A-->$1>, <B-->D>) ==> <$1-->D>>.
                 * <(&&, <A-->C>, <B-->D>) ==> <C-->D>>?
                 * 📌要点：可能两边各有「需要被替换」的地方
                 */
                // * 🚩建立映射：var2 -> term1 @ term2
                // elimination
                map_2.put(var_2, to_be_unified_1.clone());
                // * 🚩尝试消除「共同变量」
                if status.is_common_variable(var_2) {
                    // * 🚩建立映射：var2 -> term1 @ term2
                    map_1.put(var_2, to_be_unified_1.clone());
                }
                true
            }
            // * 🚩均非变量
            [None, None] => match [to_be_unified_1.as_compound(), to_be_unified_2.as_compound()] {
                // * 🚩都是复合词项⇒尝试深入
                [Some(compound_1), Some(compound_2)] if is_same_type => {
                    // * 🚩替换前提：容器相似（大小相同、像占位符位置相同）
                    if !is_same_kind_compound(compound_1, compound_2) {
                        return false;
                    }
                    // * 🚩复制词项列表 | 实际上只需拷贝其引用
                    // * 📝【2024-07-10 14:53:16】随机打乱不影响内部值，也不影响原有排序
                    let mut list = compound_1.clone_component_refs();
                    // * 🚩可交换⇒打乱
                    // * 📝from Wang：需要让算法（对两个词项）的时间复杂度为定值（O(n)而非O(n!)）
                    // * ⚠️全排列的技术难度：多次尝试会修改映射表，需要多次复制才能在检验的同时完成映射替换
                    //    * 💭【2024-07-10 14:50:09】这意味着较大的计算成本
                    // * ✨现将`rng`外置：用于在「递归深入」中产生新随机数，增强算法随机性并仍保证宏观确定性
                    let mut rng = StdRng::seed_from_u64(shuffle_rng_seed);
                    if compound_1.is_commutative() {
                        list.shuffle(&mut rng);
                        // ! 边缘情况：   `<(*, $1, $2) --> [$1, $2]>` => `<(*, A, A) --> [A]>`
                        // ! 边缘情况：   `<<A --> [$1, $2]> ==> <A --> (*, $1, $2)>>`
                        // ! 　　　　　+  `<A --> [B, C]>` |- `<A --> (*, B, C)>`✅
                        // ! 　　　　　+  `<A --> [B]>` |- `<A --> (*, B, B)>`❌
                    }
                    // * 🚩按位置逐一遍历
                    // * ✨【2024-07-10 15:02:10】更新机制：不再是「截断性返回」而是「逐个尝试」
                    //    * ⚠️与OpenNARS的核心区别：始终遍历所有子项，而非「一个不符就返回」
                    (list.into_iter().zip(compound_2.components.iter()))
                        // * 🚩逐个尝试归一化
                        .map(|(inner1, inner2)| {
                            find_unification_sub(
                                status,
                                [inner1, inner2],
                                [map_1, map_2],
                                rng.next_u64(),
                            )
                        })
                        // * 🚩非惰性迭代：只有「所有子项均能归一化」才算「能归一化」
                        //   * ⚠️不允许改为`all`：此处须强制遍历完所有子项（用`fold`+`BitAnd`）
                        //   * 📝Rust中`bool | bool`也算合法：非惰性迭代，保证「有副作用的bool函数」正常起效
                        .fold(true, BitAnd::bitand)
                }
                // * 🚩其它情况
                _ => to_be_unified_1 == to_be_unified_2, // for atomic constant terms
            },
        }
    }
    // 记录「根部坐标」从根部开始
    find_unification_sub(
        &status,
        [to_be_unified_1, to_be_unified_2],
        [map_1, map_2],
        shuffle_rng_seed,
    )
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
fn has_unification(
    var_type: &str,
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> bool {
    // 📄 `return findSubstitute(type, term1, term2, new HashMap<Term, Term>(), new HashMap<Term, Term>());`
    find_unification(
        var_type,
        to_be_unified_1,
        to_be_unified_2,
        // 创建一个临时的「变量替换映射」
        &mut VarSubstitution::new(),
        &mut VarSubstitution::new(),
        shuffle_rng_seed,
    )
}
/// 🆕【对外接口】查找独立变量归一方式
pub fn has_unification_i(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> bool {
    has_unification(
        VAR_INDEPENDENT,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

/// 🆕【对外接口】查找非独变量归一方式
pub fn has_unification_d(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> bool {
    has_unification(
        VAR_DEPENDENT,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

/// 🆕【对外接口】查找查询变量归一方式
pub fn has_unification_q(
    to_be_unified_1: &Term,
    to_be_unified_2: &Term,
    shuffle_rng_seed: u64,
) -> bool {
    has_unification(
        VAR_QUERY,
        to_be_unified_1,
        to_be_unified_2,
        shuffle_rng_seed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::AResult;
    use crate::{ok, test_term as term};
    use nar_dev_utils::macro_once;
    use rand::Rng;

    /// 测试/变量替换
    #[test]
    fn apply_substitute() -> AResult {
        fn test(substitution: &VarSubstitution, mut term: Term, expected: Term) {
            let mut compound = term
                .as_compound_mut()
                .expect("传入的不是复合词项，无法进行替换");
            compound.apply_substitute(substitution);
            assert_eq!(term, expected);
        }
        // 映射表
        let substitution = VarSubstitution::from_pairs([
            (term!("var_word"), term!("word")),
            (term!("$1"), term!("1")),
            (term!("?1"), term!("(/, A, <lock --> swan>, _, [1])")), // 变量⇒复合词项（实际情况不出现）
            (term!("[#1]"), term!("<X --> (*, Y, [Z])>")), // 复合词项⇒复合词项（实际情况不出现）
        ]);
        macro_once! {
            // * 🚩模式：待替换词项, 替换 => 替换后词项
            macro test(
                $(
                    $term_str:expr, $substitution:expr
                    => $substituted_str:expr
                )*
            ) {
                $(
                    test(&substitution, term!($term_str), term!($substituted_str));
                )*
            }
            // * 🚩一般复合词项
            "(&&, A, var_word)", substitution => "(&&, A, word)"
            "(&&, var_word, A)", substitution => "(&&, word, A)"
            "(&&, A, var_word, B)", substitution => "(&&, A, word, B)"
            "(&&, var_word, A, B)", substitution => "(&&, word, A, B)"
            // * 🚩陈述
            "<A --> var_word>", substitution => "<A --> word>"
            "<var_word --> A>", substitution => "<word --> A>"
            "<A <-> var_word>", substitution => "<A <-> word>"
            "<var_word <-> A>", substitution => "<word <-> A>"
            "<A ==> var_word>", substitution => "<A ==> word>"
            "<var_word ==> A>", substitution => "<word ==> A>"
            "<A --> $1>", substitution => "<A --> 1>"
            "<$1 --> A>", substitution => "<1 --> A>"
            "<$1 --> var_word>", substitution => "<1 --> word>"
            "<var_word --> $1>", substitution => "<word --> 1>"
            // * 🚩多层复合词项
            "<<$1 --> A> ==> <B --> $1>>", substitution => "<<1 --> A> ==> <B --> 1>>"
            "<<$1 --> var_word> ==> <var_word --> $1>>", substitution => "<<1 --> word> ==> <word --> 1>>"
            "<<var_word --> A> ==> [#1]>", substitution => "<<word --> A> ==> <X --> (*, Y, [Z])>>"
            "(--, (&&, (||, (&, (|, (*, ?1))))))", substitution => "(--, (&&, (||, (&, (|, (*, (/, A, <lock --> swan>, _, [1])))))))"
        }
        ok!()
    }

    /// 测试 / unify_find | Unification::apply_to_term | Unification::apply_to
    #[test]
    fn unify() -> AResult {
        let mut rng = StdRng::from_seed([0; 32]);
        fn test(
            mut term1: Term,
            mut term2: Term,
            var_type: &str,
            expected_1: Term,
            expected_2: Term,
            shuffle_rng: &mut impl Rng,
        ) {
            print!("unify: {term1}, {term2} =={var_type}=> ",);
            unify_find(var_type, &term1, &term2, shuffle_rng.next_u64())
                .apply_to_term(&mut term1, &mut term2);
            println!("{term1}, {term2}");
            assert_eq!(term1, expected_1);
            assert_eq!(term2, expected_2);
        }
        macro_once! {
            macro test(
                $(
                    $term_str1:expr, $term_str2:expr
                    => $var_type:expr =>
                    $substituted_str1:expr, $substituted_str2:expr
                )*
            ) {
                $(
                    test(
                        term!($term_str1),
                        term!($term_str2),
                        $var_type,
                        term!($substituted_str1),
                        term!($substituted_str2),
                        &mut rng // 用上预置的随机生成器
                    );
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
            // * ✅【2024-07-10 14:59:26】已解决：在「逐个查找替换」的「复合词项递归深入」中，不应「一不符合就截断式返回」
            //   * 📝每次「查找映射替换」均会改变「替换映射」，而「循环过程中途返回」会影响后续词项的替换
            //   * 📌【2024-07-10 15:00:45】目前认定：这三种例子均应成功
            "(*, $i, #d, ?q)", "(*, I, D, Q)" => "$" => "(*, I, #d, ?q)", "(*, I, D, Q)"
            "(*, $i, #d, ?q)", "(*, I, D, Q)" => "#" => "(*, $i, D, ?q)", "(*, I, D, Q)"
            "(*, $i, #d, ?q)", "(*, I, D, Q)" => "?" => "(*, $i, #d, Q)", "(*, I, D, Q)"

            // 多元复合词项（有序）：按顺序匹配 //
            "(*, $c, $b, $a)", "(*, (--, C), <B1 --> B2>, A)" => "$" => "(*, (--, C), <B1 --> B2>, A)", "(*, (--, C), <B1 --> B2>, A)"
               "<(*, <A-->C>, <B-->$2>) ==> <C-->$2>>", "<(*, <A-->$1>, <B-->D>) ==> <$1-->D>>"
            => "$"
            => "<(*, <A-->C>, <B-->D>) ==> <C-->D>>", "<(*, <A-->C>, <B-->D>) ==> <C-->D>>"

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

    #[test]
    fn rename_variables() -> AResult {
        fn test(mut term: Term, expected: Term) {
            // 解析构造词项
            print!("{term}");
            // 重命名变量
            let mut compound = term.as_compound_mut().expect("非复合词项，无法重命名变量");
            compound.rename_variables();
            println!("=> {term}");
            // 比对
            assert_eq!(term, expected);
        }
        macro_once! {
            // * 🚩模式：词项字符串 ⇒ 预期词项字符串
            macro test($($term:literal => $expected:expr )*) {
                $(
                    test(term!($term), term!($expected));
                )*
            }
            // 简单情况（一层） //
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
            // 不同变量名称，数值不会重复
            "(*, $A, $B, $C)" => "(*, $1, $2, $3)"
            "(*, #A, #B, #C)" => "(*, #1, #2, #3)"
            "(*, ?A, ?B, ?C)" => "(*, ?1, ?2, ?3)"
            // 不同变量类型，数值不会重复
            "(*, $A, #A, ?A)" => "(*, $1, #2, ?3)"
            // 复合词项：递归深入
            "(*, A, $B, [C, #D])" => "(*, A, $1, [C, #2])"
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

    #[test]
    fn loop_substitute() {}
}
