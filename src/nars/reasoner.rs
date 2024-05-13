//! 🎯复刻OpenNARS `nars.main_nogui.ReasonerBatch`
//! * 🚩此处扶正为[`Reasoner`]而非「批处理」
//!   * 📌更【基础】的类，名称应该更短
//!

use crate::global::ClockTime;
use crate::inference::ReasonContext;
use crate::io::{InputChannel, OutputChannel};
use crate::storage::{Memory, MemoryRecorder};
use anyhow::Result;
use nar_dev_utils::list;
use narsese::conversion::string::impl_lexical::format_instances::FORMAT_ASCII;
use navm::cmd::Cmd;

/// 模拟`ReasonerBatch`
///
/// # 📄OpenNARS
///
/// 🈚
pub trait Reasoner: ReasonContext + Sized {
    // TODO: 复刻功能

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
        // * 🚩处理输入：遍历所有通道，拿到指令
        if self.__walking_steps() == 0 {
            // * 🚩遍历所有通道，拿到要执行的指令（序列）
            let mut input_cmds = self.handle_inputs();
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
                let (run, cmds) = channel_in.next_input(self);
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
            Cmd::RES { target } => self.reset(),
            // * 🚩Narsese：输入任务（但不进行推理）
            Cmd::NSE(narsese) => {
                if let Ok(task) = self.parse_task(narsese) {
                    // * 🚩解析成功⇒记忆区输入任务
                    self.memory_mut().input_task(task);
                } else {
                    // * 🚩解析失败⇒新增输出
                    // TODO: ❓【2024-05-13 10:39:19】日志系统可能要从「记忆区」移出到「推理器」，「推理上下文」也是
                    let output = Output::ERROR {
                        description: format!(
                            "Narsese任务解析错误：{}",
                            FORMAT_ASCII.format(&narsese)
                        ),
                    };
                    self.memory_mut().recorder_mut().put(output);
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
            Cmd::REM { comment } => (),
            // * 🚩退出⇒处理完所有输出后直接退出
            Cmd::EXI { reason } => {
                // * 🚩最后的提示性输出
                self.memory_mut().recorder_mut().put(Output::INFO {
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
                self.memory_mut().recorder_mut().put(output);
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
}

mod parse_task {
    use super::*;
    use crate::{
        entity::{
            BudgetValueConcrete, Sentence, SentenceConcrete, SentenceType, ShortFloat,
            TaskConcrete, TruthValue,
        },
        inference::BudgetFunctions,
        io::symbols::JUDGMENT_MARK,
        language::Term,
        nars::DEFAULT_PARAMETERS,
    };
    use narsese::lexical::{Sentence as LexicalSentence, Task as LexicalTask};

    pub trait ReasonerParseTask: Reasoner {
        /// 模拟`StringParser.parseTask`
        /// * 🚩直接模仿`parseTask`而非`parseExperience`
        /// * 📌结合自身信息的「词法折叠」
        /// * 📝OpenNARS在解析时可能会遇到「新词项⇒新建概念」的情形
        ///   * 🚩因此需要`&mut self`
        #[doc(alias = "parse_experience")]
        fn parse_task(&mut self, narsese: LexicalTask) -> Result<Self::Task> {
            /* 📄OpenNARS源码：
            StringBuffer buffer = new StringBuffer(s);
            Task task = null;
            try {
                String budgetString = getBudgetString(buffer);
                String truthString = getTruthString(buffer);
                String str = buffer.toString().trim();
                int last = str.length() - 1;
                char punctuation = str.charAt(last);
                Stamp stamp = new Stamp(time);
                TruthValue truth = parseTruth(truthString, punctuation);
                Term content = parseTerm(str.substring(0, last), memory);
                Sentence sentence = new Sentence(content, punctuation, truth, stamp);
                if ((content instanceof Conjunction) && Variable.containVarD(content.getName())) {
                    sentence.setRevisable(false);
                }
                BudgetValue budget = parseBudget(budgetString, punctuation, truth);
                task = new Task(sentence, budget);
            } catch (InvalidInputException e) {
                String message = "ERR: !!! INVALID INPUT: parseTask: " + buffer + " --- " + e.getMessage();
                System.out.println(message);
                // showWarning(message);
            }
            return task; */
            // * 🚩判断是要被解析为「判断」还是「问题」
            let is_judgement = narsese.sentence.punctuation == JUDGMENT_MARK;
            // * 🚩生成默认真值与默认预算值
            let float = <<Self::Truth as TruthValue>::E as ShortFloat>::from_float;
            let zero = <<Self::Truth as TruthValue>::E as ShortFloat>::ZERO;
            let truth_default_values = match is_judgement {
                true => [
                    float(DEFAULT_PARAMETERS.default_judgement_frequency),
                    float(DEFAULT_PARAMETERS.default_judgement_confidence),
                ],
                // * 🚩【2024-05-13 09:44:32】目前「问题」没有真值，所以全部取`0`当占位符
                false => [zero, zero],
            };
            let default_budget = [zero, zero, zero];

            // * 📌因为OpenNARS中「前后解析依赖」，所以总需要解构——真值→预算值，词项→语句→任务
            let LexicalTask { budget, sentence } = narsese;
            let LexicalSentence {
                term,
                punctuation,
                stamp,
                truth,
            } = sentence;

            // 解析词项
            let term = Term::try_from(term)?;

            // 解析语句
            let stamp_current_serial = self.get_stamp_current_serial();
            let stamp_time = self.clock();
            let truth_is_analytic = DEFAULT_PARAMETERS.default_truth_analytic;
            let sentence_revisable = !(term.instanceof_conjunction() && term.contain_var_d());
            let sentence = <Self::Sentence as SentenceConcrete>::from_lexical(
                sentence,
                truth_default_values,
                truth_is_analytic,
                stamp_current_serial,
                stamp_time,
                sentence_revisable,
            )?;

            // 解析预算值
            use SentenceType::*;
            let (priority, durability, quality) = match sentence.punctuation() {
                Judgement(truth) => (
                    float(DEFAULT_PARAMETERS.default_judgement_priority),
                    float(DEFAULT_PARAMETERS.default_judgement_durability),
                    <Self::Budget as BudgetFunctions>::truth_to_quality(truth),
                ),
                Question => (
                    float(DEFAULT_PARAMETERS.default_question_priority),
                    float(DEFAULT_PARAMETERS.default_question_durability),
                    ShortFloat::ONE,
                ),
            };
            let budget = <Self::Budget as BudgetValueConcrete>::new(priority, durability, quality);

            // 构造任务
            let task = <Self::Task as TaskConcrete>::from_input(sentence, budget);

            // 返回
            Ok(task)
        }
    }
    impl<T: Reasoner> ReasonerParseTask for T {}
}
use navm::output::Output;
use parse_task::*;

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
