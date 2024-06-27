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

use nar_dev_utils::list;
use narsese::api::NarseseValue;
use navm::{cmd::Cmd, output::Output};

use crate::{
    control::{ReasonContext, Reasoner},
    entity::Task,
    global::ClockTime,
    inference::Budget,
    util::ToDisplayAndBrief,
};

impl Reasoner {
    /* 时钟相关 */

    /// 获取时钟时间
    pub fn time(&self) -> ClockTime {
        self.clock
    }

    pub fn init_timer(&mut self) {
        self.set_timer(0);
    }

    pub fn tick_timer(&mut self) {
        self.timer += 1;
    }

    pub fn timer(&self) -> usize {
        self.timer
    }

    pub fn set_timer(&mut self, timer: usize) {
        self.timer = timer;
    }
}

/// 推理器时钟控制
impl Reasoner {
    /// # 📄OpenNARS
    ///
    /// Start the inference process
    pub fn run(&mut self) {
        self.running = true;
    }

    /// # 📄OpenNARS
    ///
    /// Will carry the inference process for a certain number of steps
    pub fn walk(&mut self, steps: usize) {
        self.walking_steps = steps;
    }

    /// # 📄OpenNARS
    ///
    /// Will stop the inference process
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// 推理器步进
impl Reasoner {
    /// 推理器步进
    pub fn tick(&mut self) {
        // ! ❌【2024-06-27 21:06:41】不实现有关`DEBUG`的部分
        // if DEBUG {
        //     self.handle_debug();
        // }
        self.handle_input();
        self.handle_output();
        self.handle_work_cycle();
    }

    /// 处理输入
    pub fn handle_input(&mut self) {
        // * 🚩处理输入：遍历所有通道，拿到指令
        if self.walking_steps == 0 {
            // * 🚩遍历所有通道，拿到要执行的指令（序列）
            let input_cmds = self.fetch_cmd_from_input();
            // * 🚩在此过程中执行指令，相当于「在通道中调用`textInputLine`」
            for cmd in input_cmds {
                self.input_cmd(cmd);
            }
        }
    }

    /// 处理输出
    pub fn handle_output(&mut self) {
        let outputs = list![
            {output}
            while let Some(output) = (self.recorder.take())
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

    pub fn handle_work_cycle(&mut self) {
        if self.running || self.walking_steps > 0 {
            // * 🚩处理时钟
            self.clock += 1;
            self.tick_timer();
            // * 🚩工作周期
            self.work_cycle();
            // * 🚩步数递减
            if self.walking_steps > 0 {
                self.walking_steps -= 1;
            }
        }
    }
}

/// 工作周期
impl Reasoner {
    pub fn work_cycle(&mut self) {
        self.report(Output::COMMENT {
            content: format!("--- {} ---", self.time()),
        });

        // * 🚩本地任务直接处理 阶段 * //
        let no_result = self.process_direct();

        // * 🚩内部概念高级推理 阶段 * //
        // * 📝OpenNARS的逻辑：一次工作周期，只能在「直接推理」与「概念推理」中选择一个
        if no_result {
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

    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    pub fn input_cmd(&mut self, cmd: Cmd) {
        match cmd {
            // Cmd::SAV { target, path } => todo!(),
            // Cmd::LOA { target, path } => todo!(),
            // * 🚩重置：推理器复位
            Cmd::RES { .. } => self.reset(),
            // * 🚩Narsese：输入任务（但不进行推理）
            Cmd::NSE(narsese) => {
                match self.parse_task(narsese) {
                    Ok(task) => {
                        // * 🚩解析成功⇒输入任务
                        // * 🚩【2024-05-17 16:28:53】现在无需输入任务
                        self.input_task(task);
                    }
                    Err(e) => {
                        // * 🚩解析失败⇒新增输出
                        let output = Output::ERROR {
                            description: format!("Narsese任务解析错误：{e}",),
                        };
                        self.report(output);
                    }
                }
            }
            // Cmd::NEW { target } => todo!(),
            // Cmd::DEL { target } => todo!(),
            // * 🚩工作周期：添加「预备循环计数」
            Cmd::CYC(cycles) => self.walk(cycles),
            // * 🚩音量：设置音量
            Cmd::VOL(volume) => self.silence_value = volume,
            // Cmd::REG { name } => todo!(),
            // Cmd::INF { source } => todo!(),
            // Cmd::HLP { name } => todo!(),
            // * 🚩【2024-05-13 12:21:37】注释：不做任何事情
            Cmd::REM { .. } => (),
            // * 🚩退出⇒处理完所有输出后直接退出
            Cmd::EXI { reason } => {
                // * 🚩最后的提示性输出
                self.report(Output::INFO {
                    message: format!("NARust exited with reason {reason:?}"),
                });
                // * 🚩处理所有输出
                self.handle_output();
                // * 🚩最终退出程序
                std::process::exit(0);
            }
            // Cmd::Custom { head, tail } => todo!(),
            // * 🚩未知指令⇒输出提示
            _ => {
                // * 🚩解析失败⇒新增输出
                let output = Output::ERROR {
                    description: format!("未知的NAVM指令：{}", cmd),
                };
                self.report(output);
            }
        }
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
    pub fn input_task(&mut self, task: Task) {
        let budget_threshold = self.parameters.budget_threshold;
        if task.budget_above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            let narsese = NarseseValue::from_task(task.to_lexical());
            self.report(Output::IN {
                content: format!("!!! Perceived: {}", task.to_display_long()),
                narsese: Some(narsese),
            });
            // * 📝只追加到「新任务」里边，并不进行推理
            self.add_new_task(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.report(Output::COMMENT {
                content: format!("!!! Neglected: {}", task.to_display_long()),
            });
        }
    }

    /// 吸收「推理上下文」
    /// * 🚩【2024-05-21 23:18:55】现在直接调用「推理上下文」的对应方法，以便享受多分派
    pub fn absorb_context(&mut self, context: impl ReasonContext) {
        // * 🚩直接调用
        context.absorbed_by_reasoner(self);
    }
}
