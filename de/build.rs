fn main() {
    glib_build_tools::compile_resources(
        &["src/resources"],
        "resources.gresource.xml",
        "resources.gresource",
    );
}
