//! 基于「推理器」「推理上下文」有关「推理周期」的操作
//! * 🎯从「记忆区」中解耦分离
//! * 🎯在更「现代化」的同时，也使整个过程真正Rusty
//!   * 📌【2024-05-15 01:38:39】至少，能在「通过编译」的条件下复现OpenNARS
//! * 🎯将其中有关「推理周期」的代码摘录出来
//!   * 📌工作周期 from 推理器
//!   * 📌吸收推理上下文(新)
//! * 🚩【2024-05-17 21:35:04】目前直接基于「推理器」而非「记忆区」
//! * ⚠️【2024-05-18 01:25:09】目前这里所参考的「OpenNARS源码」已基本没有「函数对函数」的意义
//!   * 📌许多代码、逻辑均已重构重组
//!
//! ## Logs
//!
//! * ✅【2024-05-12 16:10:24】基本从「记忆区」迁移完所有功能
//! * ♻️【2024-06-26 11:59:58】开始根据改版OpenNARS重写

use crate::{
    control::Reasoner, entity::Task, global::ClockTime, inference::Budget, util::ToDisplayAndBrief,
};
use nar_dev_utils::list;
use navm::cmd::Cmd;

/// 推理器步进
impl Reasoner {
    /* 时钟相关 */

    /// 获取时钟时间
    pub fn time(&self) -> ClockTime {
        self.clock
    }

    /// 推理循环
    /// * 🚩只负责推理，不处理输入输出
    ///   * 📌在「处理输入」的同时，也可能发生「推理循环」（`CYC`指令）
    pub fn cycle(&mut self, steps: usize) {
        for _ in 0..steps {
            self.handle_work_cycle();
        }
    }

    /// 处理输入输出
    /// * 🚩负责处理输入输出，并**有可能触发推理循环**
    ///   * 📌输入的`CYC`指令 会【立即】触发工作周期
    ///   * 💭【2024-06-29 01:41:03】这样的机制仍有其必要性
    ///     * 💡不同通道的指令具有执行上的优先级
    ///     * 💡每个操作都是【原子性】的，执行过程中顺序先后往往影响最终结果
    pub fn handle_io(&mut self) {
        // * 🚩处理输入（可能会有推理器步进）
        self.handle_input();
        // * 🚩处理输出
        self.handle_output();
    }

    /// 处理输入：遍历所有通道，拿到指令
    fn handle_input(&mut self) {
        // * 🚩遍历所有通道，拿到要执行的指令（序列）
        let input_cmds = self.fetch_cmd_from_input();
        // * 🚩在此过程中执行指令，相当于「在通道中调用`textInputLine`」
        for cmd in input_cmds {
            self.input_cmd(cmd);
        }
    }

    /// 处理输出
    pub(super) fn handle_output(&mut self) {
        let outputs = list![
            {output}
            while let Some(output) = (self.take_output())
        ];
        if !outputs.is_empty() {
            // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
            let mut channels = list![
                {channel} // * ⚠️注意：此时顺序是倒过来的
                while let Some(channel) = (self.io_channels.output_channels.pop()) // * 此处挪出
                if (!channel.need_remove()) // * 此处筛除
            ];
            // * 🚩逆序纠正
            channels.reverse();
            // * 🚩遍历（并可引用自身）
            for channel_out in channels.iter_mut() {
                // * 🚩在此过程中解读输出
                channel_out.next_output(/* self,  */ &outputs);
            }
            // * 🚩放回
            self.io_channels.output_channels.extend(channels);
        }
    }

    fn handle_work_cycle(&mut self) {
        // * 🚩处理时钟
        self.clock += 1;
        // * 🚩工作周期
        self.work_cycle();
    }
}

/// 工作周期
impl Reasoner {
    fn work_cycle(&mut self) {
        self.report_comment(format!("--- {} ---", self.time()));

        // * 🚩本地任务直接处理 阶段 * //
        let has_result = self.process_direct();

        // * 🚩内部概念高级推理 阶段 * //
        // * 📝OpenNARS的逻辑：一次工作周期，只能在「直接推理」与「概念推理」中选择一个
        if !has_result {
            self.process_reason();
        }

        // * 🚩最后收尾 阶段 * //
        // * 🚩原「清空上下文」已迁移至各「推理」阶段
        // ! ❌不复刻「显示呈现」相关功能
    }

    /// 从输入通道中拿取一个[NAVM指令](Cmd)
    fn fetch_cmd_from_input(&mut self) -> Vec<Cmd> {
        let mut input_cmds = vec![];
        // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
        let mut channels = list![
            {channel} // * ⚠️注意：此时顺序是倒过来的
            while let Some(channel) = (self.io_channels.input_channels.pop()) // * 此处挪出
            if (!channel.need_remove()) // * 此处筛除
        ];
        // * 🚩逆序纠正
        channels.reverse();
        // * 🚩遍历（并可引用自身）
        let mut reasoner_should_run = false;
        for channel_in in channels.iter_mut() {
            // * 📝Java的逻辑运算符也是短路的——此处使用预先条件以避免运算
            // * ❓这是否意味着，一次只有一个通道能朝OpenNARS输入
            if !reasoner_should_run {
                let (run, cmds) = channel_in.next_input(/* self */);
                reasoner_should_run = run;
                // * 🆕直接用其输出扩展
                // * 💭但实际上只有一次
                input_cmds.extend(cmds);
            }
        }
        // * 🚩放回
        self.io_channels.input_channels.extend(channels);
        // * 🚩返回
        input_cmds
    }

    /// 模拟改版`Reasoner.inputTask`
    /// * 🚩【2024-05-07 22:51:11】在此对[`Budget::budget_above_threshold`](crate::inference::Budget::budget_above_threshold)引入[「预算阈值」超参数](crate::control::Parameters::budget_threshold)
    /// * 🚩【2024-05-17 15:01:06】自「记忆区」迁移而来
    ///
    /// # 📄OpenNARS
    ///
    /// Input task processing. Invoked by the outside or inside environment.
    /// Outside: StringParser (input); Inside: Operator (feedback). Input tasks
    /// with low priority are ignored, and the others are put into task buffer.
    ///
    /// @param task The input task
    pub(super) fn input_task(&mut self, task: Task) {
        let budget_threshold = self.parameters.budget_threshold;
        if task.budget_above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            self.report_in(&task);
            // * 📝只追加到「新任务」里边，并不进行推理
            self.derivation_datas.add_new_task(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.report_comment(format!("!!! Neglected: {}", task.to_display_long()));
        }
    }

    // ! 🚩【2024-06-28 00:09:12】方法「吸收推理上下文」不再需要被「推理器」实现
    // * 📌原因：现在「推理上下文」内置「推理器」的引用

    // * ℹ️【2024-08-10 14:58:02】有关「输入指令」的代码参见 `cmd_dispatch`模块
}
