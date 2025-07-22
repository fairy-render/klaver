use klaver_vm::Options;

#[derive(Default)]
pub struct Builder {
    opts: Options,
}

impl Builder {
    pub async fn build(self) -> klaver_vm::Result<Vm> {
        let vm = self
            .opts
            .global::<klaver_wintercg::WinterCG>()
            .build()
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
