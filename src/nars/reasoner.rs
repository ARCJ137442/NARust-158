//! 🎯复刻OpenNARS `nars.main_nogui.ReasonerBatch`
//! * 🚩此处扶正为[`Reasoner`]而非「批处理」
//!   * 📌更【基础】的类，名称应该更短
//!

use nar_dev_utils::list;
use navm::cmd::Cmd;

use crate::global::ClockTime;
use crate::inference::ReasonContext;
use crate::io::{InputChannel, OutputChannel};
use crate::storage::{Memory, MemoryRecorder};

/// 模拟`ReasonerBatch`
///
/// # 📄OpenNARS
///
/// 🈚
pub trait Reasoner: ReasonContext + Sized {
    // TODO: 复刻功能

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
    fn memory(&self) -> &Self::Memory;
    /// [`Reasoner::memory`]的可变版本
    fn memory_mut(&mut self) -> &mut Self::Memory;

    /// 模拟`ReasonerBatch.inputChannels`
    /// * 🚩可变
    /// * 🚩【2024-05-13 00:20:08】此处模仿OpenNARS做法，但使用`Box<dyn 特征>`实现动态分发
    ///
    /// # 📄OpenNARS
    ///
    fn input_channels(&self) -> &Vec<Box<dyn InputChannel<Reasoner = Self>>>;
    /// [`Reasoner::input_channels`]的可变版本
    fn input_channels_mut(&mut self) -> &mut Vec<Box<dyn InputChannel<Reasoner = Self>>>;

    /// 模拟`ReasonerBatch.outputChannels`
    /// * 🚩可变
    /// * 🚩【2024-05-13 00:20:08】此处模仿OpenNARS做法，但使用`Box<dyn 特征>`实现动态分发
    ///
    /// # 📄OpenNARS
    ///
    fn output_channels(&self) -> &Vec<Box<dyn OutputChannel<Reasoner = Self>>>;
    /// [`Reasoner::output_channels`]的可变版本
    fn output_channels_mut(&mut self) -> &mut Vec<Box<dyn OutputChannel<Reasoner = Self>>>;

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
        // ! ❌无需`Stamp.init();`——没有`currentSerial`
    }

    /// 模拟`ReasonerBatch.addInputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn add_input_channel(&mut self, channel: Box<dyn InputChannel<Reasoner = Self>>) {
        self.input_channels_mut().push(channel);
    }

    /// 模拟`ReasonerBatch.addOutputChannel`
    /// * ⚠️若使用`impl XChannel`会出现生命周期问题
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn add_output_channel(&mut self, channel: Box<dyn OutputChannel<Reasoner = Self>>) {
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
        let mut input_cmds = vec![];
        // * 🚩处理输入：遍历所有通道，拿到指令
        if self.__walking_steps() == 0 {
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
                    let (run, cmds) = channel_in.next_input(self);
                    reasoner_should_run = run;
                    // * 🆕直接用其输出扩展
                    // * 💭但实际上只有一次
                    input_cmds.extend(cmds);
                }
            }
            // * 🚩放回
            self.input_channels_mut().extend(channels);
            // * 🚩在此过程中执行指令，相当于「在通道中调用`textInputLine`」
            for cmd in input_cmds.into_iter() {
                self.input_cmd(cmd);
            }
        }
        // * 🚩处理输出：先取出所有输出（顺带清空），再逐个广播到所有「输出通道」
        let outputs = list![
            {output}
            while let Some(output) = (self.memory_mut().recorder_mut().take())
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
                channel_out.next_output(self, &outputs);
            }
            // * 🚩放回
            self.output_channels_mut().extend(channels);
        }
        // * 🚩最后的收尾、递进工作：在此过程中可能开始推理周期，需要一定的
        if self.__running() || self.__walking_steps() > 0 {
            *self.__clock_mut() += 1;
            self.tick_timer();
            // self.memory_mut().work_cycle(self.__clock());
            // TODO: 现在`work_cycle`被放在「推理上下文」中；后续逻辑需要重写
            if self.__walking_steps() > 0 {
                *self.__walking_steps_mut() -= 1;
            }
        }
    }

    /// 模拟`ReasonerBatch.textInputLine`
    /// * 🚩🆕【2024-05-13 02:27:07】从「字符串输入」变为「NAVM指令输入」
    #[doc(alias = "text_input_line")]
    fn input_cmd(&mut self, cmd: Cmd) {
        todo!("// TODO: 有待实装")
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
}

/// [`Reasoner`]的「具体」版本
/// * 🎯包括完全假定（字段）的构造函数
pub trait ReasonerConcrete: Reasoner + Sized {
    /// 🆕完全参数初始化
    /// * 🎯统一使用「默认实现」定义OpenNARS中的函数
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
    fn __new(name: String) -> Self;

    /// 🆕当无参初始化时的默认名称
    const DEFAULT_NAME: &'static str = "Reasoner";

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
    ///
    /// # 📄OpenNARS
    ///
    /// 🈚
    #[inline]
    fn with_name(name: &str) -> Self {
        Self::__new(name.into())
    }
}
