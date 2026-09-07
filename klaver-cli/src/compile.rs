use std::path::Path;

use color_eyre::eyre::{bail, eyre};
#[cfg(feature = "swc")]
use klaver_modules::loaders::{SwcCompiler, SwcCompilerOptions, SwcDecocators};

#[cfg(feature = "oxc")]
use klaver_modules::loaders::{OxcCompiler, OxcCompilerOptions};

#[allow(unreachable_code)]
pub fn compile(path: impl AsRef<Path>) -> color_eyre::Result<Vec<u8>> {
    #[cfg(feature = "swc")]
    {
        let compiler = SwcCompiler::new_with(SwcCompilerOptions {
            decorators: SwcDecocators::Legacy,
            ..Default::default()
        });
        let result = compiler
            .compile(path.as_ref())
            .map_err(|err| eyre!("{}", err))?;
        println!("Compiled code:\n{}", String::from_utf8_lossy(&result.code));
        return Ok(result.code);
    }

    #[cfg(feature = "oxc")]
    {
        use color_eyre::eyre::eyre;

        let compiler = OxcCompiler::new_with(OxcCompilerOptions {
            legacy_decorators: true,
            ..Default::default()
        });
        let result = compiler
            .compile(path.as_ref())
            .map_err(|err| eyre!("{}", err))?;

        return Ok(result.code);
    }

    bail!("No compiler")
}
