use klaver_modules::{
    loaders::FileLoader,
    resolvers::{FileResolver, ResolveOptions},
};
use klaver_vm::{Options, RuntimeError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RuntimeError> {
    let work_dir = std::env::current_dir()
        .expect("current dir")
        .canonicalize()
        .expect("canonicalize");

    let mut file_loader = FileLoader::default();
    file_loader.add_transformer(());

    let vm = Options::default()
        .module::<klaver_test::TestModule>()
        .resolver(FileResolver::new_with(work_dir, ResolveOptions::default()))
        .loader(file_loader)
        .build()
        .await?;

    vm.run_module("./klaver-test/examples/test.js").await?;

    Ok(())
}
