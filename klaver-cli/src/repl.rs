use klaver_vm::Vm;
use reedline::{DefaultPrompt, Reedline, Signal};

#[derive(clap::Args)]
pub struct ReplCmd {}

impl ReplCmd {
    pub async fn run(&self, vm: Vm) -> color_eyre::Result<()> {
        let prompt = DefaultPrompt::default();
        let mut line_editor = Reedline::create();

        loop {
            let sig = line_editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    println!("We processed: {}", buffer);
                }
                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                }
                x => {
                    println!("Event: {:?}", x);
                }
            }
        }
        Ok(())
    }
}
