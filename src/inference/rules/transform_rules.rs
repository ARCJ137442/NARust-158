//! 转换推理/转换规则

use crate::{
    control::{
        ContextDerivationConcept, ReasonContext, ReasonContextTransform, ReasonContextWithLinks,
        ReasonDirection,
    },
    entity::{Sentence, TLink, TruthValue},
    inference::BudgetInferenceContext,
    io::symbols::INHERITANCE_RELATION,
    language::{CompoundTermRef, Term},
    util::RefCount,
};
use nar_dev_utils::unwrap_or_return;

/// 推理引擎「转换推理」的唯一入口
/// * 📝【2024-05-20 11:46:32】在「直接推理」之后、「概念推理」之前使用
///
/// # 📄OpenNARS
///
/// The TaskLink is of type TRANSFORM,
/// and the conclusion is an equivalent transformation
pub fn transform_task(context: &mut ReasonContextTransform) {
    // * 🚩预处理 | 📌【2024-06-07 23:12:34】断定其中的「tLink」就是「当前任务链」
    let t_link = context.current_task_link();
    let task_rc = t_link.target_rc();
    let task = task_rc.get_();
    debug_assert!(
        task.content().is_compound(),
        "// ! 此处必须假定其为复合词项：转换规则的任务均为复合词项"
    );
    let task_content = unwrap_or_return! {
        ?task.content().as_compound()
    };
    let indexes = t_link.indexes();

    // * 🚩获取有待转换的「继承」陈述（引用）
    let inh = unwrap_or_return! {
        ?unwrap_or_return! {
            ?get_inheritance_to_be_transform(
                task_content,
                indexes
            )
        // * 🚩提取出了继承项⇒开始转换
        // * 🚩【2024-07-03 11:35:40】修改：传入时复制
        }.as_statement_type(INHERITANCE_RELATION)
    };
    // * 🚩拷贝词项以隔离修改
    let old_content = task_content.inner.clone();
    let inheritance_to_be_transform = inh.statement.clone();
    drop(task);
    drop(task_rc);

    // * 🚩预先分派 @ 转换的是整体
    match old_content == inheritance_to_be_transform {
        // * 🚩待转换词项为整体（自身）⇒特别分派（可能转换多次）
        true => {
            // * 🚩需要重新拿到「陈述引用」
            let inh = unwrap_or_return!(?inheritance_to_be_transform.as_statement());
            let inh_subject = inh.subject;
            let inh_predicate = inh.predicate;
            // * 🚩转换前项
            if let Some(inh_subject) = inh_subject.as_compound() {
                transform_subject_product_image(inh_subject, inh_predicate, context);
            }
            // * 🚩转换后项
            if let Some(inh_predicate) = inh_predicate.as_compound() {
                transform_predicate_product_image(inh_subject, inh_predicate, context);
            }
        }
        // * 🚩其它⇒转换内部的「继承」系词
        // * 📌【2024-07-05 18:22:05】此处不传递indexes，避免借用冲突
        false => transform_product_image(inheritance_to_be_transform, old_content, context),
    }
}

/// 🆕获取【需要参与后续「转换」操作】的「继承」陈述
fn get_inheritance_to_be_transform<'t>(
    task_content: CompoundTermRef<'t>,
    indexes: &[usize],
) -> Option<&'t Term> {
    match indexes.len() {
        // * 🚩本身是乘积 | <(*, term, #) --> #>
        2 if task_content.instanceof_inheritance() => Some(task_content.inner),
        // * 🚩乘积在蕴含里边 | <<(*, term, #) --> #> ==> #>
        3 => task_content.component_at(indexes[0]),
        // * 🚩乘积在蕴含的条件中 | <(&&, <(*, term, #) --> #>, #) ==> #>
        4 => task_content
            .as_conditional()?
            // * 🚩提取其中的条件项
            .1
            // * 🚩按索引提取「条件」中的继承陈述
            .component_at(indexes[1]),
        // * 🚩其它⇒返回
        _ => None,
    }
}

/// # 📄OpenNARS
///
/// Equivalent transformation between products and images
/// {<(*, S, M) --> P>, S@(*, S, M)} |- <S --> (/, P, _, M)>
/// {<S --> (/, P, _, M)>, P@(/, P, _, M)} |- <(*, S, M) --> P>
/// {<S --> (/, P, _, M)>, M@(/, P, _, M)} |- <M --> (/, P, S, _)>
fn transform_product_image(
    inheritance_to_be_transform: Term,
    old_content: Term,
    context: &mut ReasonContextTransform,
) {
    // * 🚩提取参数 * //
    let t_link = context.current_task_link();
    let task_rc = t_link.target_rc();
    let task = task_rc.get_();
    let indexes = t_link.indexes();
    let reason_direction = context.reason_direction();

    // * 🚩词项 * //
    // * 📝此处针对各类「条件句」等复杂逻辑
    let new_inheritance =
        unwrap_or_return!(?transform_inheritance(inheritance_to_be_transform, indexes));

    // * 🚩用新构造的「继承」产生【在替换旧有内容中替换之后的】新词项
    let content =
        unwrap_or_return!(?replaced_transformed_content(old_content, indexes, new_inheritance));

    // * 🚩真值 * //
    let truth = task.get_truth().map(TruthValue::from);

    // * 🚩预算 * //
    drop(task);
    drop(task_rc);
    use ReasonDirection::*;
    let budget = match reason_direction {
        // * 🚩复合前向 | 此处无需unwrap：预算推理处再断言
        Forward => context.compound_forward(truth.as_ref(), &content),
        // * 🚩复合反向
        Backward => context.compound_backward(&content),
    };

    // * 🚩结论 * //
    // * 📝「真值」在「导出任务」时（从「当前任务」）自动生成
    context.single_premise_task_structural(content, truth, budget);
}

fn replaced_transformed_content(
    old_content: Term,
    indexes: &[usize],
    new_inheritance: Term,
) -> Option<Term> {
    todo!()
}

fn transform_inheritance(inheritance_to_be_transform: Term, indexes: &[usize]) -> Option<Term> {
    let inheritance_statement = inheritance_to_be_transform
        .as_statement()
        .expect("【2024-07-05 18:51:25】已在传参前断言");
    todo!()
}

fn transform_subject_product_image(
    inh_subject: CompoundTermRef,
    inh_predicate: &Term,
    context: &mut ReasonContextTransform,
) {
    todo!()
}

fn transform_predicate_product_image(
    inh_subject: &Term,
    inh_predicate: CompoundTermRef,
    context: &mut ReasonContextTransform,
) {
    todo!()
}
