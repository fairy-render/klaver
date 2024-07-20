use std::process::Stdio;

use rquickjs::{class::Trace, function::Rest, ArrayBuffer, Class, Ctx, Result};

pub type Module = js_shell_mod;

klaver::module_info!("@klaver/shell" => Module);

#[rquickjs::module(rename_vars = "camelCase")]
pub mod shell_mod {

    use futures::TryStreamExt;
    use rquickjs::{function::Rest, Class, Ctx, Object, Result};

    use klaver_streams::{async_byte_iterator, AsyncByteIterError};

    use super::Exec;
    pub use super::Pipe;

    #[rquickjs::function]
    pub async fn cat<'js>(ctx: Ctx<'js>, path: String) -> Result<Object<'js>> {
        let file = tokio::fs::File::open(path).await?;
        let stream = tokio_util::io::ReaderStream::new(file);
        async_byte_iterator(
            ctx,
            stream
                .map_ok(|b| b.to_vec())
                .map_err(|_| AsyncByteIterError),
        )
    }

    #[rquickjs::function]
    pub fn sh<'js>(
        ctx: Ctx<'js>,
        cmd: rquickjs::String<'js>,
        Rest(args): Rest<rquickjs::String<'js>>,
    ) -> Result<Class<'js, Exec<'js>>> {
        Ok(Class::instance(ctx, Exec { cmd, args })?)
    }
}

#[derive(Trace, Clone)]
#[rquickjs::class]
pub struct Pipe<'js> {
    cmds: Vec<Exec<'js>>,
}

impl<'js> Pipe<'js> {
    async fn run(&self) -> Result<tokio::process::Child> {
        let Some((first, rest)) = self.cmds.split_first() else {
            panic!("no exec")
        };

        let first = first.build_cmd()?.stdout(Stdio::piped()).spawn()?;

        let mut children = vec![first];

        let _len = rest.len();

        for (_i, next) in rest.iter().enumerate() {
            let prev: Stdio = children
                .last_mut()
                .unwrap()
                .stdout
                .take()
                .expect("")
                .try_into()?;

            let child = next
                .build_cmd()?
                .stdin(prev)
                .stdout(Stdio::piped())
                .spawn()?;

            children.push(child);
        }

        let last = children.pop().expect("last");

        for mut child in children {
            child.wait().await?;
        }

        Ok(last)
    }
}

#[rquickjs::methods]
impl<'js> Pipe<'js> {
    #[qjs(constructor)]
    pub fn new(Rest(args): Rest<Class<'js, Exec<'js>>>) -> Result<Pipe<'js>> {
        Ok(Pipe {
            cmds: args
                .into_iter()
                .map(|m| m.try_borrow().map(|m| m.clone()))
                .collect::<Result<_>>()?,
        })
    }

    pub fn pipe(&self, ctx: Ctx<'js>, exec: Class<'js, Exec<'js>>) -> Result<Class<'js, Self>> {
        let mut cloned = self.clone();
        cloned.cmds.push(exec.try_borrow()?.clone());
        Class::instance(ctx, cloned)
    }

    pub async fn output(&self, ctx: Ctx<'js>) -> Result<rquickjs::ArrayBuffer<'js>> {
        let child = self.run().await?;

        let output = child.wait_with_output().await?;

        ArrayBuffer::new(ctx, output.stdout)
    }
}

#[derive(Trace, Clone)]
#[rquickjs::class]
pub struct Exec<'js> {
    cmd: rquickjs::String<'js>,
    args: Vec<rquickjs::String<'js>>,
}

#[rquickjs::methods]
impl<'js> Exec<'js> {
    #[qjs(constructor)]
    pub fn new(cmd: rquickjs::String<'js>, Rest(args): Rest<rquickjs::String<'js>>) -> Exec<'js> {
        Exec { cmd, args }
    }

    pub fn pipe(
        &self,
        ctx: Ctx<'js>,
        exec: Class<'js, Exec<'js>>,
    ) -> Result<Class<'js, Pipe<'js>>> {
        Class::instance(
            ctx,
            Pipe {
                cmds: vec![self.clone(), exec.try_borrow()?.clone()],
            },
        )
    }

    pub async fn output(&self) -> Result<String> {
        let status = self.build_cmd()?.output().await?;
        Ok(String::from_utf8(status.stdout).unwrap())
    }
}

impl<'js> Exec<'js> {
    // pub fn from(cmd: &str) -> Exec {
    //     let mut cmds = cmd.split(' ').map(|m| m.to_string());
    //     let cmd = cmds.next().expect("cmd").to_string();
    //     let args = cmds.collect();
    //     Exec { cmd, args }
    // }

    // pub fn new(cmd: String, args: Vec<String>) -> Exec {
    //     Exec { cmd, args }
    // }

    fn build_cmd(&self) -> Result<tokio::process::Command> {
        let mut cmd = tokio::process::Command::new(&self.cmd.to_string()?);

        let args = self
            .args
            .iter()
            .map(|m| m.to_string())
            .collect::<Result<Vec<_>>>()?;

        cmd.args(args);

        Ok(cmd)
    }
}
