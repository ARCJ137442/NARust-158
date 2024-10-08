//! 用于NARS推理器的「超参数」
//! * 🆕不再是全局常量，而是可随推理器而变的结构体
//!
//! ## logs
//!
//! * ♻️【2024-09-05 01:22:19】现移至模块根部，以统领全局超参数
//!   * ℹ️理由：避免让`control`模块与其它模块耦合——让`entity`、`storage`与之解耦

use crate::global::Float;
use nar_dev_utils::macro_once;
use serde::{Deserialize, Serialize};

/// 用于决定推理器诸多推理中的「k值」
/// * 🚩🆕【2024-05-03 16:00:14】根据在「真值函数」中的实际用途，此处将其修改为「浮点数」[`Float`]类型
pub type EvidentialHorizon = Float;

macro_once! {
    // * 🚩模式：自动为「属性 = 值」生成[`Default`]实现
    macro parameters(
        // 结构体定义（包括文档注释）
        $(#[$struct_attr:meta])*
        pub struct $struct_name:ident {
            $( // 一条属性，包括文档注释
                $(#[$attr:meta])*
                $v:vis $name:ident : $type:ty = $default:expr
            ),* $(,)?
        }
    ) {
        // 结构体定义
        $(#[$struct_attr])*
        pub struct $struct_name {
            $(
                $(#[$attr])*
                // #[serde(default = concat!("default_values::", $name))]
                // ! ❌【2024-09-02 17:22:32】无法在宏展开后的属性宏中使用宏展开结果
                //   * ℹ️期望`concat`先展开，实则先输入进了`#[serde(default = ...)]`中
                //   * ❌也无法使用「展开成`#[serde]`的宏」：展开前被提前代入，语法错误
                $v $name: $type,
            )*
        }
        /// 所有字段的默认值
        #[doc(hidden)]
        mod default_values {
            use super::*;
            $(
                // 作为常量函数的「默认值函数」
                #[doc(hidden)]
                pub const fn $name() -> $type { $default }
            )*
        }
        /// 内部功能
        impl $struct_name {
            /// 实现「常量化默认函数」
            /// * 🎯构建自身，并直接可作为`const`常量
            ///   * ✅兼用于实现[`Default`]
            pub const fn default_const() -> Self {
                Self {
                    $(
                        // 使用模块路径下的函数
                        $name: default_values::$name(),
                    )*
                }
            }
        }
        /// 实现[`Default`]
        impl Default for $struct_name {
            #[inline]
            fn default() -> Self {
                // 直接使用「常量函数」
                Self::default_const()
            }
        }
    }
    /// NARS运行的「超参数」
    ///
    /// # 📄OpenNARS
    ///
    /// Collected system parameters. To be modified before compiling.
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Parameters {
        /// # 📄OpenNARS
        ///
        /// Concept decay rate in ConceptBag, in [1, 99].
        pub concept_forgetting_cycle: usize = 10,

        /// # 📄OpenNARS
        ///
        /// TaskLink decay rate in TaskLinkBag, in [1, 99].
        pub task_link_forgetting_cycle: usize = 20,

        /// # 📄OpenNARS
        ///
        /// TermLink decay rate in TermLinkBag, in [1, 99].
        pub term_link_forgetting_cycle: usize = 50,

        /// # 📄OpenNARS
        ///
        /// Silent threshold for task reporting, in [0, 100].
        pub silent_level: usize = 0,

        /// # 📄OpenNARS
        ///
        /// Task decay rate in TaskBuffer, in [1, 99].
        pub new_task_forgetting_cycle: usize = 1,

        /// # 📄OpenNARS
        ///
        /// Maximum TermLinks checked for novelty for each TaskLink in TermLinkBag
        pub max_matched_term_link: usize = 10,

        /// # 📄OpenNARS
        ///
        /// Maximum TermLinks used in reasoning for each Task in Concept
        pub max_reasoned_term_link: usize = 3,

        /// * 📝亦即NAL推理中的「k值」
        /// * 🚩🆕【2024-05-03 16:01:12】现在「k值」真正变成了浮点类型
        /// # 📄OpenNARS
        ///
        /// Evidential Horizon, the amount of future evidence to be considered.
        pub horizon: EvidentialHorizon = 1.0, // or 2, can be float

        /// # 📄OpenNARS
        ///
        /// Reliance factor, the empirical confidence of analytical truth.
        pub reliance: Float = 0.9, // the same as default confidence

        /// # 📄OpenNARS
        ///
        /// The budget threshold rate for task to be accepted.
        pub budget_threshold: Float = 0.01,


        /// 用于各类「预期达标」的阈值
        /// * 📝仅在OpenNARS 3.x中使用
        ///   * 📄目标预期
        ///
        /// # 📄OpenNARS
        ///
        /// Default expectation for confirmation.
        pub default_confirmation_expectation: Float = 0.8,

        /// 用于「任务缓冲区」的「新任务/新近任务」筛选
        /// * 📌目前使用「创建の预期」而非「确认の预期」
        ///   * 📄OpenNARS 1.5.8中即使用此参数
        ///   * 📝在OpenNARS 3.x中用于「创建预期」，对应词项`TRUE`/`FALSE`
        ///
        /// # 📄OpenNARS
        ///
        /// Default expectation for confirmation.
        pub default_creation_expectation: Float = 0.66,

        /// 🆕默认的「输入真值分析性」
        /// * 🎯减少来自`TruthValue`的硬编码
        pub default_truth_analytic: bool = false,

        /// 🆕默认的「输入真值频率」
        /// * 🎯减少来自`StringParser`的硬编码
        pub default_judgement_frequency: Float = 1.0,

        /// # 📄OpenNARS
        ///
        /// Default confidence of input judgement.
        pub default_judgement_confidence: Float = 0.9,

        /// # 📄OpenNARS
        ///
        /// Default priority of input judgement
        pub default_judgement_priority: Float = 0.8,

        /// # 📄OpenNARS
        ///
        /// Default durability of input judgement
        pub default_judgement_durability: Float = 0.8,

        /// # 📄OpenNARS
        ///
        /// Default priority of input question
        pub default_question_priority: Float = 0.9,

        /// # 📄OpenNARS
        ///
        /// Default durability of input question
        pub default_question_durability: Float = 0.9,

        /// # 📄OpenNARS
        ///
        /// Level granularity in Bag, two digits
        pub bag_level: usize = 100,

        /// # 📄OpenNARS
        ///
        /// Level separation in Bag, one digit, for display (run-time adjustable) and management (fixed)
        pub bag_threshold: usize = 10,

        /// # 📄OpenNARS
        ///
        /// Hash table load factor in Bag
        pub load_factor: Float = 0.5,

        /// # 📄OpenNARS
        ///
        /// Size of ConceptBag
        pub concept_bag_size: usize = 1000,

        /// # 📄OpenNARS
        ///
        /// Size of TaskLinkBag
        pub task_link_bag_size: usize = 20,

        /// # 📄OpenNARS
        ///
        /// Size of TermLinkBag
        pub term_link_bag_size: usize = 100,

        /// # 📄OpenNARS
        ///
        /// Size of TaskBuffer
        pub task_buffer_size: usize = 10,

        /// # 📄OpenNARS改版
        ///
        /// 🆕Initial priority of a new Concept
        pub concept_initial_priority: Float = 0.01,

        /// # 📄OpenNARS改版
        ///
        /// 🆕Initial durability of a new Concept
        pub concept_initial_durability: Float = 0.01,

        /// # 📄OpenNARS改版
        ///
        /// 🆕Initial quality of a new Concept
        pub concept_initial_quality: Float = 0.01,

        /// # 📄OpenNARS
        ///
        /// Maximum length of Stamp, a power of 2
        pub maximum_stamp_length: usize = 8,

        /// # 📄OpenNARS
        ///
        /// Remember recently used TermLink on a Task
        pub term_link_record_length: usize = 10,

        /// # 📄OpenNARS
        ///
        /// Maximum number of beliefs kept in a Concept
        pub maximum_belief_length: usize = 7,

        /// # 📄OpenNARS
        ///
        /// Maximum number of goals kept in a Concept
        pub maximum_questions_length: usize = 5,

        /// 🆕新近任务袋容量
        /// * 🎯【2024-09-02 17:29:22】分离自「概念袋遗忘周期」
        ///   * 📌默认值数据来自「概念袋」
        #[serde(default = "default_values::novel_task_bag_size")] // ! 使用宏自动生成的默认值，以便向下兼容
        pub novel_task_bag_size: usize = 1000,

        /// 🆕新近任务的遗忘周期
        /// * 🎯【2024-09-02 17:29:22】分离自「概念袋遗忘周期」
        ///   * 📌默认值数据来自「概念袋」
        #[serde(default = "default_values::novel_task_forgetting_cycle")]
        pub novel_task_forgetting_cycle: usize = 10,
    }
}

/// 🆕全局、默认的「超参数」
/// * 🎯用于各特征的默认实现
/// * 🚩【2024-05-04 01:31:58】不如就利用这个「全局常量」暂且在代码逻辑中「做死编码」
///   * ⚡通过「硬编码」的方式减少传参，提升开发效率
///     * 📄无需在某些函数中浪费时间与精力「到处传参」，特别是[`w2c`](crate::inference::UtilityFunctions::w2c)、[`c2w`](crate::inference::UtilityFunctions::c2w)
///   * ✅将「重构/整理 工作」交给后续进阶项目开发
///     * 📌【2024-05-04 01:34:20】目前工作中心仍然是「复现/复刻」而非「探索」
///     * 📌仍旧以「开发效率」为首要指标
pub const DEFAULT_PARAMETERS: Parameters = Parameters::default_const();

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use nar_dev_utils::asserts;

    /// 测试/对应性
    /// * 🎯默认值是否与OpenNARS一一对应
    #[test]
    fn test_default_value() {
        // 获取默认值
        let parameters = DEFAULT_PARAMETERS;
        // 验证默认值
        macro_once! {
            // * 🚩模式：`键 => 预期值`
            macro test_default_value($(
                $field:ident => $expected:expr
            )*) {
                // * 🚩检查整个结构体
                let expected = Parameters {
                    $( $field: $expected ),*
                };
                assert_eq!(parameters, expected);
                // * 🚩逐一检查预期值
                $(
                    assert_eq!(parameters.$field, $expected);
                )*
            }
            // 默认值表
            concept_forgetting_cycle         => 10
            task_link_forgetting_cycle       => 20
            term_link_forgetting_cycle       => 50
            silent_level                     => 0
            new_task_forgetting_cycle        => 1
            max_matched_term_link            => 10
            max_reasoned_term_link           => 3
            horizon                          => 1.0
            reliance                         => 0.9
            budget_threshold                 => 0.01
            default_confirmation_expectation => 0.8
            default_creation_expectation     => 0.66
            default_truth_analytic           => false
            default_judgement_frequency      => 1.0
            default_judgement_confidence     => 0.9
            default_judgement_priority       => 0.8
            default_judgement_durability     => 0.8
            default_question_priority        => 0.9
            default_question_durability      => 0.9
            bag_level                        => 100
            bag_threshold                    => 10
            load_factor                      => 0.5
            concept_bag_size                 => 1000
            task_link_bag_size               => 20
            term_link_bag_size               => 100
            task_buffer_size                 => 10
            concept_initial_priority         => 0.01
            concept_initial_durability       => 0.01
            concept_initial_quality          => 0.01
            maximum_stamp_length             => 8
            term_link_record_length          => 10
            maximum_belief_length            => 7
            maximum_questions_length         => 5
            novel_task_bag_size              => 1000
            novel_task_forgetting_cycle      => 10
        }
    }

    /// 测试/一致性
    /// * 🎯测试两个`default`是否一致
    #[test]
    fn test_default_consistency() {
        asserts! {
            // 两个「默认」构造函数一致
            Parameters::default_const() == Parameters::default()
            // 「常量默认」构造函数与「常量」一致
            Parameters::default_const() == DEFAULT_PARAMETERS
        }
    }
}
