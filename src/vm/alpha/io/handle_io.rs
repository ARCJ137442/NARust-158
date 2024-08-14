//! 处理虚拟机的输入输出

use crate::vm::alpha::RuntimeAlpha;
use nar_dev_utils::list;
use navm::cmd::Cmd;

impl RuntimeAlpha {
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

    /// 处理输出
    pub(in super::super) fn handle_output(&mut self) {
        let outputs = list![
            {output}
            while let Some(output) = (self.reasoner.take_output())
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
}
