//! 推理器 定义
//! * 🎯以Rust特征定义「推理器」
//! * 🚩此处扶正为[`Reasoner`]而非「批处理」
//!   * 📌更【基础】的类，名称应该更短
//! * 📄在OpenNARS 3.x中已更名为 `nars.main.NAR`

use super::*;
use crate::entity::{BudgetValue, Item, ShortFloat, TaskConcrete};
use crate::global::ClockTime;
use crate::inference::{DerivationContextDirect, DerivationContextReason, ReasonContext};
use crate::io::{InputChannel, OutputChannel};
use crate::nars::{Parameters, DEFAULT_PARAMETERS};
use crate::storage::{BagConcrete, Memory, NovelTaskBag};
use crate::ToDisplayAndBrief;
use nar_dev_utils::list;
use narsese::api::NarseseValue;
use navm::cmd::Cmd;
use navm::output::Output;
use std::collections::VecDeque;

/// 模拟`ReasonerBatch`
/// * 🚩【2024-05-17 16:48:52】现在直接就是「具体类型」，并且采用泛型而非「关联类型」的方法
///   * ⚠️避免在绑定「推导上下文」类型中遇到一堆「类型重绑定」
///     * 📄如`Task = Self::Task`
/// * 🚩【2024-05-17 17:11:34】因为`parse_task`需要[`Sized`]
///
/// # 📄OpenNARS
///
/// 🈚
pub trait Reasoner<C: ReasonContext>: Sized {
    /// 绑定的「记录者」类型
    /// * 🚩【2024-05-17 14:57:13】迁移自原「记忆区」的「记录者」
    ///   * 🎯能在「推导上下文」构建之前完成「报告/输出」的工作
    type Recorder: MemoryRecorderConcrete;

    /// 模拟`Memory.recorder`、`getRecorder`、`setRecorder`
    /// * 🚩🆕【2024-05-07 20:08:35】目前使用新定义的[`MemoryRecorder`]类型
    /// * 📝OpenNARS中`Memory`用到`recorder`的地方：`init`、`inputTask`、`activatedTask`
    ///
    /// # 📄OpenNARS
    ///
    /// Inference record text to be written into a log file
    fn recorder(&self) -> &Self::Recorder;
    /// [`Memory::recorder`]的可变版本
    fn recorder_mut(&mut self) -> &mut Self::Recorder;

    /// 🆕缓存一条「推理输出」
    /// * 📌功能迁移自OpenNARS`Memory.report`
    /// * 📝【2024-05-17 16:27:34】对原先【堆在「记忆区」处】的报告作分离处理
    ///   * 🚩一些「报告」直接在推理器处存储即可
    ///   * 🚩另一些「在推理过程中积累的新报告」则交由后续「吸收上下文」时处理
    #[inline(always)]
    fn report(&mut self, output: Output) {
        self.recorder_mut().put(output);
    }

    /// 绑定的「推导上下文」类型
    type DerivationContextReason: DerivationContextReason<C>;
    /// 绑定的「直接推理上下文」类型
    type DerivationContextDirect: DerivationContextDirect<C, Target = Self::DerivationContextReason>;

    /// 模拟`Stamp.currentSerial`
    /// * 📝OpenNARS中要保证「每个新创的时间戳都有一个序列号，且这个序列号唯一」
    /// * ⚠️同一个时间也可能有多个时间戳被创建
    /// * ❌【2024-05-13 10:02:00】拒绝全局静态变量
    fn __stamp_current_serial(&mut self) -> &mut ClockTime;
    /// 🆕简化对[`Reasoner::__stamp_current_serial`]的调用
    /// * 📝OpenNARS中「先自增，再使用」
    fn get_stamp_current_serial(&mut self) -> ClockTime {
        *self.__stamp_current_serial() += 1;
        *self.__stamp_current_serial()
    }

    // ! ❌暂不复刻`DEBUG`：除打印消息外无用（实际上与「新的 日志/输出 系统」理由类似）

    /// 模拟`ReasonerBatch.name`
    /// * 📝推理器名称
    ///   * 💭正好对上NAVM指令`NEW`
    /// * 🚩只读
    ///
    /// # 📄OpenNARS
    ///
    /// The name of the reasoner
    fn name(&self) -> &str;

    /// 模拟`ReasonerBatch.memory`
    /// * 🚩可变
    ///
    /// # 📄OpenNARS
    ///
    /// The memory of the reasoner
    fn memory(&self) -> &C::Memory;
    /// [`Reasoner::memory`]的可变版本
    fn memory_mut(&mut self) -> &mut C::Memory;

    /// 模拟`ReasonerBatch.inputChannels`
    /// * 🚩可变
    /// * 🚩【2024-05-13 00:20:08】此处模仿OpenNARS做法，但使用`Box<dyn 特征>`实现动态分发
    /// * 📝【2024-05-15 11:37:44】Rust中对所有特征对象都最好显式指定「对象生命周期」
    ///   * 📌目前`&self`和`&Vec`周期一致，而`dyn XXXChannel`和`'this`周期一致
    ///     * 📍这意味着：内部「通道」的生命周期，与自身结构的生命周期一致
    ///   * ❌【2024-05-15 11:38:41】不在`&self`处添加约束`'this`：`self`整个对象与「引用」的生命周期是不同的
    ///
    /// # 📄OpenNARS
    ///
    fn input_channels<'this>(
        &self,
    ) -> &Vec<Box<dyn InputChannel<Context = C /* , Reasoner = Self */> + 'this>>
    where
        Self: 'this;
    /// [`Reasoner::input_channels`]的可变版本
    fn input_channels_mut<'this>(
        &mut self,
    ) -> &mut Vec<Box<dyn InputChannel<Context = C /* , Reasoner = Self */> + 'this>>
    where
        Self: 'this;

    /// 模拟`ReasonerBatch.outputChannels`
    /// * 🚩可变
    /// * 🚩【2024-05-13 00:20:08】此处模仿OpenNARS做法，但使用`Box<dyn 特征>`实现动态分发
    ///
    /// # 📄OpenNARS
    ///
    fn output_channels<'this>(
        &self,
    ) -> &Vec<Box<dyn OutputChannel<Context = C /* , Reasoner = Self */> + 'this>>
    where
        Self: 'this;
    /// [`Reasoner::output_channels`]的可变版本
    fn output_channels_mut<'this>(
        &mut self,
    ) -> &mut Vec<Box<dyn OutputChannel<Context = C /* , Reasoner = Self */> + 'this>>
    where
        Self: 'this;

    /// 模拟`ReasonerBatch.clock`、`ReasonerBatch.getTime`
    /// * 🚩读取公有，修改私有
    ///
    /// # 📄OpenNARS
    ///
    /// System clock, relatively defined to guarantee the repeatability of behaviors
    #[doc(alias = "time")]
    fn clock(&self) -> ClockTime;
    /// [`Reasoner::time`]的可变版本（私有）
    #[doc(alias = "__time_mut")]
    fn __clock_mut(&mut self) -> &mut ClockTime;

    /// 模拟`ReasonerBatch.timer`、`ReasonerBatch.getTimer`
    /// * 🚩读取公有，修改私有
    /// * 🚩【2024-05-13 00:15:49】目前挪到前边来，将与「时钟」有关的都放一起
    ///
    /// # 📄OpenNARS `timer`
    ///
    /// System clock - number of cycles since last output
    ///
    /// # 📄OpenNARS `getTimer`
    ///
    /// @return System clock : number of cycles since last output
    fn timer(&self) -> usize;
    /// 模拟`ReasonerBatch.setTimer`
    /// * 📌[`Reasoner::timer`]的可变版本（私有）
    ///
    /// # 📄OpenNARS `setTimer`
    ///
    /// set System clock : number of cycles since last output
    fn __timer_mut(&mut self) -> &mut usize;

    /// 模拟`ReasonerBatch.walkingSteps`
    /// * 🚩私有
    /// * 🚩【2024-05-13 00:15:49】目前挪到前边来，将与「时钟」有关的都放一起
    /// * 📝实际上是一个基于「预备要推理循环次数」的「缓存量」
    ///   * 🚩接收各处的[`Reasoner::walk`]调用
    ///   * 🚩随后在统一的[`Reasoner::tick`]中执行
    ///
    /// # 📄OpenNARS
    ///
    fn __walking_steps(&self) -> usize;
    /// [`Reasoner::__walking_steps`]的可变版本（私有）
    fn __walking_steps_mut(&mut self) -> &mut usize;

    /// 模拟`ReasonerBatch.running`
    /// * 🚩私有
    ///
    /// # 📄OpenNARS
    ///
    /// Flag for running continuously
    fn __running(&self) -> bool;
    /// [`Reasoner::__running`]的可变版本（私有）
    fn __running_mut(&mut self) -> &mut bool;

    /// 模拟`ReasonerBatch.finishedInputs`、`ReasonerBatch.isFinishedInputs`
    /// * 🚩读取公有，修改私有
    ///
    /// # 📄OpenNARS
    ///
    /// determines the end of {@link NARSBatch} program (set but not accessed in this class)
    fn finished_inputs(&self) -> bool;
    /// [`Reasoner::finished_inputs`]的可变版本（私有）
    fn __finished_inputs_mut(&mut self) -> &mut bool;

    /// 模拟`ReasonerBatch.silenceValue`
    /// * 🚩读取公有，修改私有
    /// * 🚩【2024-05-13 00:18:23】此处不用「原子值」，暂不考虑多线程场景
    ///
    /// # 📄OpenNARS
    ///
    fn silence_value(&self) -> usize;
    /// [`Reasoner::silence_value`]的可变版本（私有）
    fn __silence_value_mut(&mut self) -> &mut usize;

    /*================构造函数================*/

    /// 模拟`ReasonerBatch.reset`
    ///
    /// # 📄OpenNARS
    ///
    fn reset(&mut self) {
        /* 📄OpenNARS源码：
        running = false;
        walkingSteps = 0;
        clock = 0;
        memory.init();
        Stamp.init();
        // timer = 0; */
        *self.__running_mut() = false;
        *self.__walking_steps_mut() = 0;
        *self.__clock_mut() = 0;
        self.memory_mut().init();
        // 添加记录
        self.recorder_mut().put(Output::INFO {
            message: "-----RESET-----".into(),
        });
        *self.__stamp_current_serial() = 0;
    }

    /// 模拟`ReasonerBatch.addInputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn add_input_channel(
        &mut self,
        channel: Box<dyn InputChannel<Context = C /* , Reasoner = Self */>>,
    ) {
        self.input_channels_mut().push(channel);
    }

    /// 模拟`ReasonerBatch.addOutputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn add_output_channel<'this, 'channel: 'this>(
        &'this mut self,
        channel: Box<dyn OutputChannel<Context = C /* , Reasoner = Self */> + 'channel>,
    ) where
        Self: 'this,
    {
        self.output_channels_mut().push(channel);
    }

    // ! ❌不模拟`ReasonerBatch.removeInputChannel`
    //   * 📝OpenNARS中仅用于「请求推理器移除自身」
    //   * 🚩这实际上可以被「标记『待移除』，下次遍历到时直接删除」的方法替代
    //   * ✅同时避免了「循环引用」「动态判等」问题

    // ! ❌不模拟`ReasonerBatch.removeOutputChannel`

    /// 模拟`ReasonerBatch.run`
    ///
    /// # 📄OpenNARS
    ///
    /// Start the inference process
    #[inline]
    fn run(&mut self) {
        *self.__running_mut() = true;
    }

    /// 模拟`ReasonerBatch.stop`
    ///
    /// # 📄OpenNARS
    ///
    /// Will stop the inference process
    #[inline]
    fn stop(&mut self) {
        *self.__running_mut() = false;
    }

    /// 模拟`ReasonerBatch.walk`
    /// * 📝OpenNARS中仅设置步骤，并不立刻开始推理
    ///
    /// # 📄OpenNARS
    ///
    /// Will carry the inference process for a certain number of steps
    ///
    /// @param n The number of inference steps to be carried
    #[inline]
    fn walk(&mut self, n: usize) {
        *self.__walking_steps_mut() = n;
    }

    /// 模拟`ReasonerBatch.tick`、`ReasonerBatch.doTick`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[doc(alias = "do_tick")]
    fn tick(&mut self) {
        /* 📄OpenNARS源码：
        if (DEBUG) {
            if (running || walkingSteps > 0 || !finishedInputs) {
                System.out.println("// doTick: "
                        + "walkingSteps " + walkingSteps
                        + ", clock " + clock
                        + ", getTimer " + getTimer()
                        + "\n//    memory.getExportStrings() " + memory.getExportStrings());
                System.out.flush();
            }
        }
        if (walkingSteps == 0) {
            boolean reasonerShouldRun = false;
            for (InputChannel channelIn : inputChannels) {
                reasonerShouldRun = reasonerShouldRun
                        || channelIn.nextInput();
            }
            finishedInputs = !reasonerShouldRun;
        }
        // forward to output Channels
        ArrayList<String> output = memory.getExportStrings();
        if (!output.isEmpty()) {
            for (OutputChannel channelOut : outputChannels) {
                channelOut.nextOutput(output);
            }
            output.clear(); // this will trigger display the current value of timer in Memory.report()
        }
        if (running || walkingSteps > 0) {
            clock++;
            tickTimer();
            memory.workCycle(clock);
            if (walkingSteps > 0) {
                walkingSteps--;
            }
        } */
        // ! ❌不复刻`if (DEBUG) { ... }`
        // * 🚩处理输入：遍历所有通道，拿到指令
        if self.__walking_steps() == 0 {
            // * 🚩遍历所有通道，拿到要执行的指令（序列）
            let input_cmds = self.handle_inputs();
            // * 🚩在此过程中执行指令，相当于「在通道中调用`textInputLine`」
            for cmd in input_cmds.into_iter() {
                self.input_cmd(cmd);
            }
        }
        // * 🚩处理输出：先取出所有输出（顺带清空），再逐个广播到所有「输出通道」
        self.handle_outputs();
        // * 🚩最后的收尾、递进工作：在此过程中可能开始推理周期
        if self.__running() || self.__walking_steps() > 0 {
            self.handle_cycle();
        }
    }

    /// 🆕从`ReasonerBatch.doTick`分离出的「获取输入」逻辑
    /// * 🚩处理所有输入通道：从其中一个可用通道中拉取输入
    fn handle_inputs(&mut self) -> Vec<Cmd> {
        let mut input_cmds = vec![];
        // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
        let mut channels = list![
            {channel} // * ⚠️注意：此时顺序是倒过来的
            while let Some(channel) = (self.input_channels_mut().pop()) // * 此处挪出
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
        self.input_channels_mut().extend(channels);
        // * 🚩返回
        input_cmds
    }

    /// 🆕从`ReasonerBatch.doTick`分离出的「处理输出」逻辑
    /// * 🚩处理所有输出：全部取出，并发送到所有「输出通道」
    /// * 🎯用于复用：在程序执行「退出」指令时，仍然处理完所有输出
    fn handle_outputs(&mut self) {
        let outputs = list![
            {output}
            while let Some(output) = (self.recorder_mut().take())
        ];
        if !outputs.is_empty() {
            // * 🚩先将自身通道中的元素挪出（在此过程中筛除），再从此临时通道中计算与获取输入（以便引用自身）
            let mut channels = list![
                {channel} // * ⚠️注意：此时顺序是倒过来的
                while let Some(channel) = (self.output_channels_mut().pop()) // * 此处挪出
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
            self.output_channels_mut().extend(channels);
        }
    }

    /// 🆕从`ReasonerBatch.doTick`分离出的「运行一次推理周期」
    /// * 📝OpenNARS的逻辑：各地朝`walking_steps`设置步数，然后由[`Reasoner::tick`]统一执行
    /// * 🚩【2024-05-13 12:23:30】目前此处沿袭OpenNARS的做法
    ///
    /// TODO: 【2024-05-13 12:16:01】后续或许要重构此类设计，不能全盘照搬OpenNARS
    fn handle_cycle(&mut self) {
        if self.__walking_steps() > 0 {
            *self.__walking_steps_mut() = self.__walking_steps().saturating_sub(1); // * 🚩软性相减，减到`0`就停止
            *self.__clock_mut() += 1;
            self.tick_timer();
            // self.memory_mut().work_cycle(self.__clock());
            todo!("// TODO: 现在`work_cycle`被放在「推理上下文」中；后续逻辑需要重写");
        }
    }

    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    #[doc(alias = "text_input_line")]
    fn input_cmd(&mut self, cmd: Cmd) {
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
                        // TODO: ❓【2024-05-13 10:39:19】日志系统可能要从「记忆区」移出到「推理器」，「推理上下文」也是
                        let output = Output::ERROR {
                            description: format!("Narsese任务解析错误：{e}",),
                        };
                        self.recorder_mut().put(output);
                    }
                }
            }
            // Cmd::NEW { target } => todo!(),
            // Cmd::DEL { target } => todo!(),
            // * 🚩推理循环：添加「预备循环计数」
            Cmd::CYC(cycles) => self.walk(cycles),
            // * 🚩音量：设置音量
            Cmd::VOL(volume) => *self.__silence_value_mut() = volume,
            // Cmd::REG { name } => todo!(),
            // Cmd::INF { source } => todo!(),
            // Cmd::HLP { name } => todo!(),
            // * 🚩【2024-05-13 12:21:37】注释：不做任何事情
            Cmd::REM { .. } => (),
            // * 🚩退出⇒处理完所有输出后直接退出
            Cmd::EXI { reason } => {
                // * 🚩最后的提示性输出
                self.recorder_mut().put(Output::INFO {
                    message: format!("NARust exited with reason {reason:?}"),
                });
                // * 🚩处理所有输出
                self.handle_outputs();
                // * 🚩最终退出程序
                std::process::exit(0);
            }
            // Cmd::Custom { head, tail } => todo!(),
            // * 🚩未知指令⇒输出提示
            _ => {
                // * 🚩解析失败⇒新增输出
                // TODO: ❓【2024-05-13 10:39:19】日志系统可能要从「记忆区」移出到「推理器」，「推理上下文」也是
                let output = Output::ERROR {
                    description: format!("未知的NAVM指令：{}", cmd),
                };
                self.recorder_mut().put(output);
            }
        }
    }

    // ! ❌【2024-05-13 02:22:35】暂不模拟`toString`：OpenNARS中直接调用了记忆区，但此处或许可以更详细

    /// 模拟`ReasonerBatch.updateTimer`
    ///
    /// # 📄OpenNARS
    ///
    /// To get the timer value and then to
    /// reset it by {@link #initTimer()};
    /// plays the same role as {@link nars.gui.MainWindow#updateTimer()}
    ///
    /// @return The previous timer value
    fn update_timer(&mut self) -> usize {
        /* 📄OpenNARS源码：
        long i = getTimer();
        initTimer();
        return i; */
        let i = self.timer();
        self.init_timer();
        i
    }

    /// 模拟`ReasonerBatch.initTimer`
    ///
    /// # 📄OpenNARS
    ///
    /// Reset timer;
    /// plays the same role as {@link nars.gui.MainWindow#initTimer()}
    fn init_timer(&mut self) {
        *self.__timer_mut() = 0;
    }

    /// 模拟`ReasonerBatch.tickTimer`
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    fn tick_timer(&mut self) {
        *self.__timer_mut() += 1;
    }

    /* ---------- Long-term storage for multiple cycles ---------- */

    /// 模拟`Memory.newTasks`
    /// * 🚩读写：OpenNARS中要读写对象
    ///   * 🚩【2024-05-12 14:38:58】决议：两头都有
    ///     * 在「记忆区回收上下文」时从「上下文的『新任务』接收」
    /// * 📝虽然OpenNARS中被认作是「短期工作空间」，但实际上是个长期的工作空间
    ///   * 📝并且，只在「记忆区」内部被使用，用于「直接推理」
    ///   * 📌实际上是在「直接推理」中被取出任务（与`novel_tasks`一致）
    ///
    /// # 📄OpenNARS
    ///
    /// List of new tasks accumulated in one cycle, to be processed in the next cycle
    fn __new_tasks(&self) -> &[C::Task];
    /// [`Reasoner::__new_tasks`]的可变版本
    /// * 🚩【2024-05-07 21:13:39】暂时用[`VecDeque`]代替：需要FIFO功能
    fn __new_tasks_mut(&mut self) -> &mut VecDeque<C::Task>;

    /// 绑定的「任务袋」类型
    /// * 🚩【2024-05-07 20:04:25】必须与「概念」中的「任务」一致
    /// * 🎯对应[`Reasoner::__novel_tasks`]
    /// * 🚩【2024-05-18 11:07:56】现从「记忆区」中迁移出来
    type NovelTaskBag: NovelTaskBag<Task = C::Task>;

    /// 模拟`Memory.novelTasks`
    /// * 📌新近任务
    /// * 🚩私有+读写
    ///
    /// # 📄OpenNARS
    ///
    /// New tasks with novel composed terms, for delayed and selective processing
    fn __novel_tasks(&self) -> &Self::NovelTaskBag;
    /// [`Memory::novel_tasks`]的可变版本
    fn __novel_tasks_mut(&mut self) -> &mut Self::NovelTaskBag;

    /* ---------- new task entries ---------- */
    /*
     * There are several types of new tasks, all added into the
     * newTasks list, to be processed in the next workCycle.
     * Some of them are reported and/or logged.
     */

    /// 模拟`Memory.inputTask`
    /// * 🚩【2024-05-07 22:51:11】在此对[`BudgetValue::above_threshold`]引入[「预算阈值」超参数](crate::nars::Parameters::budget_threshold)
    /// * 🚩【2024-05-17 15:01:06】自「记忆区」迁移而来
    ///
    /// TODO: ❓后续是否有必要迁移到独立的代码中去，比如「推理器主控」中
    ///   * 🎯功能分离
    ///
    /// # 📄OpenNARS
    ///
    /// Input task processing. Invoked by the outside or inside environment.
    /// Outside: StringParser (input); Inside: Operator (feedback). Input tasks
    /// with low priority are ignored, and the others are put into task buffer.
    ///
    /// @param task The input task
    fn input_task(&mut self, task: C::Task) {
        /* 📄OpenNARS源码：
        if (task.getBudget().aboveThreshold()) {
            recorder.append("!!! Perceived: " + task + "\n");
            report(task.getSentence(), ReportType.IN); // report input
            newTasks.add(task); // wait to be processed in the next workCycle
        } else {
            recorder.append("!!! Neglected: " + task + "\n");
        } */
        let budget_threshold = DEFAULT_PARAMETERS.budget_threshold;
        // * ✅【2024-05-07 23:22:54】现在通过重命名「真值」「预算值」的相应方法，不再有命名冲突（`from_float`→`from_floats`）
        let budget_threshold = C::ShortFloat::from_float(budget_threshold);
        if task.budget().above_threshold(budget_threshold) {
            // ? 💭【2024-05-07 22:57:48】实际上只需要输出`IN`即可：日志系统不必照着OpenNARS的来
            // * 🚩此处两个输出合而为一
            let narsese = NarseseValue::from_task(task.to_lexical());
            self.recorder_mut().put(Output::IN {
                content: format!("!!! Perceived: {}", task.to_display_long()),
                narsese: Some(narsese),
            });
            // * 📝只追加到「新任务」里边，并不进行推理
            self.__new_tasks_mut().push_back(task);
        } else {
            // 此时还是输出一个「被忽略」好
            self.recorder_mut().put(Output::COMMENT {
                content: format!("!!! Neglected: {}", task.to_display_long()),
            });
        }
    }
}

/// [`Reasoner`]的「具体」版本
/// * 🎯包括完全假定（字段）的构造函数
pub trait ReasonerConcrete<C: ReasonContext>: Reasoner<C> + Sized {
    /// 🆕完全参数初始化
    /// * 🎯统一使用「默认实现」定义OpenNARS中的函数
    /// * 🚩【2024-05-15 16:40:41】现在新增「超参数」设定
    ///   * 🎯以备后续「引用解耦」
    /// * 📌会创建一个空白的「推理记录者」
    ///   * 🚩需要初始化各种「默认参数」
    ///
    /// TODO: 【2024-05-18 11:19:15】❓是否要传入「记忆区」的构造函数
    ///
    /// # 📄OpenNARS 参考源码
    ///
    /// ```java
    /// public ReasonerBatch(String name) {
    ///     this.name = name;
    ///     memory = new Memory(this);
    ///     inputChannels = new ArrayList<>();
    ///     outputChannels = new ArrayList<>();
    /// }
    /// ```
    fn __new(
        name: String,
        parameters: Parameters,
        new_tasks: VecDeque<C::Task>,
        novel_tasks: Self::NovelTaskBag,
    ) -> Self;

    /// 🆕构造一个**默认**的「新近任务（袋）」
    /// * 🚩构造一个空容器，使用自身参数
    fn __new_novel_tasks(parameters: &Parameters) -> Self::NovelTaskBag {
        BagConcrete::new(
            // * 🚩复刻`nars.storage.NovelTaskBag.capacity`
            parameters.task_buffer_size,
            // * 🚩复刻`nars.storage.NovelTaskBag.forgetRate`
            parameters.new_task_forgetting_cycle,
        )
    }

    /// 🆕当无参初始化时的默认名称
    const DEFAULT_NAME: &'static str = "NARust Reasoner";

    /// 模拟`new ReasonerBatch()`
    /// * 📌无参初始化（使用默认名称）
    /// * 🆕📜默认实现：使用默认名称
    ///   * 💭因为OpenNARS中除了「名称」未初始化外，均与「带名称初始化」一致
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn new() -> Self {
        Self::with_name(Self::DEFAULT_NAME)
    }

    /// 模拟`new ReasonerBatch(String name)`
    /// * 📌带参初始化
    /// * 🚩【2024-05-15 16:41:23】目前使用「默认参数」初始化
    ///   * 📌后续可调
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn with_name(name: impl Into<String>) -> Self {
        Self::with_name_and_parameters(name.into(), DEFAULT_PARAMETERS)
    }

    /// 🆕带参初始化（名称+超参数）
    /// * 🎯用于「名称&超参数 外其它参数的默认初始化」
    ///   * 📄在「允许自定义名称与超参数」的同时，无需传入其它「应该被默认的参数集」
    #[inline]
    fn with_name_and_parameters(name: impl Into<String>, parameters: Parameters) -> Self {
        Self::__new(
            // * 📌需要自定义的参数
            name.into(),
            parameters,
            // * 📌需要默认值的参数
            VecDeque::new(), // TODO: 🏗️【2024-05-07 21:09:58】日后是否可独立成一个`add`、`size`、`get`的特征？
            Self::__new_novel_tasks(&parameters), // * 🚩新近任务袋：空
        )
    }
}
