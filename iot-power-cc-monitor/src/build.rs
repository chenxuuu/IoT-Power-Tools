#[cfg(windows)]
fn main() {
    embed_resource::compile("./src/icon.rc", embed_resource::NONE);
}

#[cfg(unix)]
fn main() {
}
