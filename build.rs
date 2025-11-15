extern crate embed_resource;

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc");
    }
}
