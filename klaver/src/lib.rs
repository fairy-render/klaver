use klaver_vm::Options;
use rquickjs::CatchResultExt;

#[derive(Default)]
pub struct Builder {
    opts: Options,
}

impl Builder {
    pub async fn build(self) -> klaver_vm::Result<Vm> {
        let vm = self
            .opts
            .search_path(".")
            .global::<klaver_wintertc::WinterCG>()
            .build()
            .await?;

        klaver_vm::async_with!(vm => |ctx| {
            klaver_wintertc::backend::Tokio::default().set_runtime(&ctx).catch(&ctx)?;
            Ok(())
        })
        .await?;

        Ok(Vm { vm })
    }
}

pub struct Vm {
    vm: klaver_vm::Vm,
}

impl Vm {}

impl std::ops::Deref for Vm {
    type Target = klaver_vm::Vm;

    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}
