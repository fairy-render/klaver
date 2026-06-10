use klaver::Vm;
use reedline::{DefaultPrompt, Reedline, Signal};
use rquickjs::{CatchResultExt, Object, Value};

pub async fn run(vm: Vm, source: Option<&str>, exec: bool, types: bool) -> color_eyre::Result<()> {
    if let Some(source) = source {
        if exec {
            vm.async_with(async |ctx| {
                ctx.eval_promise(source)
                    .catch(&ctx)?
                    .into_future::<()>()
                    .await
                    .catch(&ctx)?;
                Ok(())
            })
            .await?;
        } else if types {
            // vm.env().typings().files().write_to(source, false).await?;
        } else {
            let path = if !(source.starts_with("../") && !source.starts_with("./")) {
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

        println!("Welcome to Klaver!");

        loop {
            let sig = line_editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(buffer)) => {
                    let ret = vm
                        .async_with(async |ctx| {
                            let ret = ctx
                                .eval_promise(buffer)
                                .catch(&ctx)?
                                .into_future::<Object<'_>>()
                                .await
                                .catch(&ctx)?;
                            let ret = ret.get::<_, Value>("value")?;
                            println!(
                                "{}",
                                klaver_core::value::format(&ctx, &ret, Default::default())?
                            );
                            Ok(())
                        })
                        .await;

                    if let Err(err) = ret {
                        eprintln!("{err}");
                    }
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
