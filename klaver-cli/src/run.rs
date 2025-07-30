use klaver_vm::Vm;
use reedline::{DefaultPrompt, Reedline, Signal};
use rquickjs::{CatchResultExt, Value};

pub async fn run(vm: Vm, source: Option<&str>, exec: bool) -> color_eyre::Result<()> {
    if let Some(source) = source {
        if exec {
            klaver_vm::async_with!(vm => |ctx| {
              ctx.eval_promise(source).catch(&ctx)?.into_future::<()>().await.catch(&ctx)?;
              Ok(())
            })
            .await?;
        } else {
            let path = if !(source.starts_with("../") || source.starts_with("./")) {
                format!("./{}", source)
            } else {
                source.to_string()
            };
            vm.run_module(&path).await?;
        }
    } else {
        let mut prompt = DefaultPrompt::default();
        prompt.left_prompt = reedline::DefaultPromptSegment::Empty;
        prompt.right_prompt = reedline::DefaultPromptSegment::Empty;
        let mut line_editor = Reedline::create();

        println!("Welcome to Klaver");

        loop {
            let sig = line_editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    let ret = klaver_vm::async_with!(vm => |ctx| {
                      let ret = ctx.eval_promise(buffer).catch(&ctx)?.into_future::<Value<'_>>().await.catch(&ctx)?;
                      println!("{}", klaver_util::format(&ctx, &ret, Default::default())?);
                      Ok(())
                    })
                    .await;

                    if let Err(err) = ret {
                        eprintln!("{err}");
                    }
                    // println!("We processed: {}", buffer);
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
    }

    Ok(())
}
