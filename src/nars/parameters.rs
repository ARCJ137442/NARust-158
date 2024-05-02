//! 用于NARS推理器的「超参数」
//! * 🆕不再是全局常量，而是可随推理器而变的结构体

use crate::global::Float;
use nar_dev_utils::macro_once;

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
                $v $name: $type,
            )*
        }
        impl $struct_name {
            /// 实现「常量化默认函数」
            /// * 🎯构建自身，并直接可作为`const`常量
            ///   * ✅兼用于实现[`Default`]
            pub const fn default_const() -> Self {
                Self {
                    $(
                        $name: $default,
                    )*
                }
            }
        }
        /// 实现[`Default`]
        impl Default for $struct_name {
            fn default() -> Self {
                // 直接使用「常量函数」
                Self::default_const()
            }
        }
    }
    /// NARS运行的「超参数」
    ///
    /// # 📄OpenNARS `nars.main_nogui.Parameters`
    ///
    /// Collected system parameters. To be modified before compiling.
    #[derive(Debug, Clone, Copy, PartialEq)]
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

        /// # 📄OpenNARS
        ///
        /// Evidential Horizon, the amount of future evidence to be considered.
        pub horizon: usize = 1, // or 2, can be float

        /// # 📄OpenNARS
        ///
        /// Reliance factor, the empirical confidence of analytical truth.
        pub reliance: Float = 0.9, // the same as default confidence

        /// # 📄OpenNARS
        ///
        /// The budget threshold rate for task to be accepted.
        pub budget_threshold: Float = 0.01,

        /// # 📄OpenNARS
        ///
        /// Default expectation for confirmation.
        pub default_confirmation_expectation: Float = 0.8,

        /// # 📄OpenNARS
        ///
        /// Default expectation for confirmation.
        pub default_creation_expectation: Float = 0.66,

        /// # 📄OpenNARS
        ///
        /// Default confidence of input judgment.
        pub default_judgment_confidence: Float = 0.9,

        /// # 📄OpenNARS
        ///
        /// Default priority of input judgment
        pub default_judgment_priority: Float = 0.8,

        /// # 📄OpenNARS
        ///
        /// Default durability of input judgment
        pub default_judgment_durability: Float = 0.8,

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
    }
}

/// 🆕全局、默认的「超参数」
/// * 🎯用于各特征的默认实现
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
            )*) {$(
                assert_eq!(parameters.$field, $expected);
            )*}
            // 默认值表
            concept_forgetting_cycle => 10
            task_link_forgetting_cycle => 20
            term_link_forgetting_cycle => 50
            silent_level => 0
            new_task_forgetting_cycle => 1
            max_matched_term_link => 10
            max_reasoned_term_link => 3
            horizon => 1
            reliance => 0.9
            budget_threshold => 0.01
            default_confirmation_expectation => 0.8
            default_creation_expectation => 0.66
            default_judgment_confidence => 0.9
            default_judgment_priority => 0.8
            default_judgment_durability => 0.8
            default_question_priority => 0.9
            default_question_durability => 0.9
            bag_level => 100
            bag_threshold => 10
            load_factor => 0.5
            concept_bag_size => 1000
            task_link_bag_size => 20
            term_link_bag_size => 100
            task_buffer_size => 10
            maximum_stamp_length => 8
            term_link_record_length => 10
            maximum_belief_length => 7
            maximum_questions_length => 5
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
