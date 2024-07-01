use relative_path::RelativePath;

pub fn check_extensions(name: &str, extensions: &[String]) -> bool {
    let path = RelativePath::new(name);
    path.extension()
        .map(|extension| {
            extensions
                .iter()
                .any(|known_extension| known_extension == extension)
        })
        .unwrap_or(false)
}
