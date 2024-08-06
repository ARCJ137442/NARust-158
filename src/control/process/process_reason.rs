//! 推理器有关「概念推理/高级推理」的功能
//! * 🎯模拟以`RuleTables.reason`为入口的「概念推理」
//!   * 📌处理概念(内部) from 工作周期
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! ## Logs
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-05-18 16:36:06】目前从「推理周期」迁移出来
//! * ♻️【2024-06-26 11:59:58】开始根据改版OpenNARS重写

use crate::{
    control::{
        ReasonContext, ReasonContextConcept, ReasonContextTransform, ReasonContextWithLinks,
        Reasoner,
    },
    entity::{Concept, Sentence, TLink, TLinkType, TaskLink, TermLink},
    util::{RefCount, ToDisplayAndBrief},
};
use nar_dev_utils::{unwrap_or_return, JoinTo};

impl Reasoner {
    /// 概念推理
    /// * 📌「概念推理」控制机制的入口函数
    pub(in crate::control) fn process_reason(&mut self) {
        // * 🚩从「直接推理」到「概念推理」过渡 阶段 * //
        // * 🚩选择概念、选择任务链、选择词项链（中间亦有推理）⇒构建「概念推理上下文」
        let context = unwrap_or_return!(?self.preprocess_concept() => ());
        // * 🚩内部概念高级推理 阶段 * //
        // * 🚩【2024-06-27 21:37:10】此处内联整个函数，以避免借用问题
        Self::process_concept(context);
    }

    /// * ✅【2024-06-28 01:29:07】现在不再需要关注「推理引擎导致借用冲突」的问题
    ///   * 💡返回之后直接使用函数指针，而函数指针是[`Copy`]类型——可以复制以脱离借用
    fn preprocess_concept(&mut self) -> Option<ReasonContextConcept> {
        // * 🚩从「记忆区」拿出一个「概念」准备推理 | 源自`processConcept`
        let mut current_concept = self.memory.take_out_concept()?;
        self.report_comment(format!("* Selected Concept: {}", current_concept.term()));

        // * 🚩预点火（实质上仍属于「直接推理」而非「概念推理」）
        let mut current_task_link = unwrap_or_return! {
            // * 🚩从「概念」拿出一个「任务链」准备推理 | 源自`Concept.fire`
            ?current_concept.take_out_task_link()
            => {
                // * 🚩中途返回时要回收
                self.memory_mut().put_back_concept(current_concept);
                None // ! 返回
            }
        };
        // * 📝此处应该是「重置信念链，以便后续拿取词项链做『概念推理』」

        // * 🚩若为「转换」类链接⇒转换推理并返回
        if current_task_link.link_type() == TLinkType::Transform {
            self.report_comment(format!(
                "* Selected TaskLink to transform: {}",
                current_task_link.to_display()
            ));
            self.process_concept_transform(current_concept, current_task_link);
            return None;
        }

        // * 🚩从选取的「任务链」获取要（分别）参与推理的「词项链」
        let belief_links_to_reason: Vec<TermLink> =
            self.choose_term_links_to_reason(&mut current_concept, &mut current_task_link);
        if belief_links_to_reason.is_empty() {
            self.report_comment(format!(
                "* Selected TaskLink without reasoning: {}",
                current_task_link.to_display()
            ));
            // * 🚩中途返回时要回收
            // ! ❓【2024-05-24 22:55:**】↓这个「当前任务链」不知为何，按理应该放回，但若放回则推不出结果
            // * 🚩【2024-05-24 22:53:16】目前「维持原判」不放回「当前任务链」
            // * 🚩【2024-06-29 00:08:44】遵照同义重构前`Concept.fire`代码 同义修复：始终需要放回「当前任务链」
            // * 📝OpenNARS在「当前概念没找到信念链」时，仍然将「已取出的『当前任务链』」放回「当前概念」中
            // 🔗https://github.com/ARCJ137442/OpenNARS-158-dev/blob/be8e7ddb9f2c918ac7c99491ef9a6f6318a93c18/src/nars/entity/Concept.java#L453
            // * 🚩回收当前任务链
            let overflowed = current_concept.put_task_link_back(current_task_link);
            if let Some(overflowed_task_link) = overflowed {
                self.report_comment(format!(
                    "!!! Overflowed TaskLink: {}",
                    overflowed_task_link.to_display_long()
                ));
            }
            // * 🚩回收当前概念
            self.memory.put_back_concept(current_concept);
            // 返回空
            return None;
        }
        // * 🚩报告
        self.report_comment(format!(
            "* Selected TaskLink: {}\n  with TermLinks:\n  + {}",
            current_task_link.to_display(),
            belief_links_to_reason
                .iter()
                .map(ToDisplayAndBrief::to_display)
                .join_to_new("\n  + ")
        ));

        // * 🚩在最后构造并返回
        let context = ReasonContextConcept::new(
            self,
            current_concept,
            current_task_link,
            belief_links_to_reason,
        );
        Some(context)
    }

    /// 🆕中途提取出的「处理转换推理」
    /// * 🚩创建上下文并独自调用推理
    fn process_concept_transform(&mut self, current_concept: Concept, current_task_link: TaskLink) {
        // * 🚩创建「转换推理上下文」
        // * ⚠️此处「当前信念链」为空，可空情况不一致，使用一个专门的「推理上下文」类型
        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * @ C="livingIn"
        // * 📄T="<{tim} --> (/,livingIn,_,{graz})>"
        // * @ C="{graz}"
        let transform_f = self.inference_engine.transform_f();
        let mut context = ReasonContextTransform::new(self, current_concept, current_task_link);
        // * 🚩交给「推理引擎」开始做「转换推理」
        transform_f(&mut context);
        // * 🚩独立吸收上下文
        context.absorbed_by_reasoner();
    }

    fn choose_term_links_to_reason(
        &mut self,
        current_concept: &mut Concept,
        current_task_link: &mut TaskLink,
    ) -> Vec<TermLink> {
        let mut to_reason_links = vec![];
        // * 🚩拿取最多「最大词项链数目」次
        for _ in 0..self.parameters.max_reasoned_term_link {
            let link = match current_concept
                .take_out_term_link_from_task_link(current_task_link, self.time())
            {
                Some(link) => link,
                None => break,
            };
            // * 🚩添加
            to_reason_links.push(link);
        }
        to_reason_links
        // * 🚧仍有「迭代器版本」作为参考
        // let time = self.time();
        // let reason_recorder = &mut self.recorder;
        // (0..self.parameters.max_reasoned_term_link)
        //     // * 🚩逐个尝试拿出
        //     .map(|_| current_concept.take_out_term_link_from_task_link(current_task_link, time))
        //     .flatten()
        //     .map(|link| {
        //         // * 🚩报告
        //         reason_recorder.put(util_outputs::output_comment(format!(
        //             "* Selected TermLink: {}",
        //             link.to_display()
        //         )));
        //         // * 🚩返还（追加）
        //         link
        //     })
        //     .collect::<Vec<_>>()
    }

    /// 具体形式有待商议（借用问题）
    fn process_concept(mut context: ReasonContextConcept) {
        // * 🚩开始推理；【2024-05-17 17:50:05】此处代码分离仅为更好演示其逻辑
        // * 📝【2024-05-19 18:40:54】目前将这类「仅修改一个变量的推理」视作一组推理，共用一个上下文
        // * 📌【2024-05-21 16:33:56】在运行到此处时，「推理上下文」的「当前信念」不在「待推理词项链表」中，但需要「被聚焦」
        loop {
            // * 🔥启动概念推理：点火！ | 此时已经预设「当前信念」「当前信念链」「新时间戳」准备完毕
            // * 🚩有当前信念 ⇒ 先尝试匹配处理
            let old_derived_tasks = context.num_new_tasks();
            if context.has_current_belief() {
                // * 🚩开始「匹配推理」
                let matching = context.core.reasoner.inference_engine.matching_f();
                matching(&mut context);
            }

            // * 🚩若作为「判断」成功⇒直接结束该信念的推理
            // * 📝尚且不能完全迁移出「概念推理」中：需要在一个「推理上下文」中行事
            let has_result = context.num_new_tasks() > old_derived_tasks;
            if has_result && context.current_task().get_().is_judgement() {
                continue;
            }
            // ! 📝此处OpenNARS原意是：若「之前通过『直接推理』或『概念推理/本地推理』获得了结果」，则不再进行下一步推理
            // * 📌依据：`long_term_stability.nal`
            // * 📄ONA中的结果有两个：
            // * 1. `Answer: <{tom} --> murder>. %1.000000; 0.729000%`
            // * 2. `<{tim} --> murder>. %1.000000; 0.810000%`
            // * 📄OpenNARS 3.1.0的结果：`Answer <{tim} --> murder>. %1.00;0.85%`
            // * 📝目前的结果是：`ANSWER: <{tim} --> murder>. %1.00;0.81% {195 : 5;7}`

            // * 🚩🆕概念推理 触发 报告
            context.report_comment(format!(
                "* Reasoning on: {} <~ {} ~> {}",
                context.current_task().get_().content(),
                context.current_concept().term(),
                &*context.current_belief_link().target()
            ));
            // * 🚩交给推理引擎做「概念推理」
            let reason_f = context.core.reasoner.inference_engine.reason_f();
            reason_f(&mut context);

            // * 🚩切换上下文中的「当前信念」「当前信念链」「新时间戳」 | 每次「概念推理」只更改「当前信念」与「当前信念链」
            let (has_next, overflowed_belief_link) = context.next_belief();
            // 汇报「溢出的信念链」
            if let Some(overflowed_belief_link) = overflowed_belief_link {
                context.report_comment(format!(
                    "!!! Overflowed belief link: {}",
                    overflowed_belief_link.to_display_long()
                ));
            }

            // * 🚩没有更多词项链⇒结束
            if !has_next {
                break;
            }
        }

        // * ✅归还「当前任务链/当前信念链」的工作已经在「吸收上下文」中被执行
        // * 🚩吸收并清空上下文
        context.absorbed_by_reasoner();
    }
}
