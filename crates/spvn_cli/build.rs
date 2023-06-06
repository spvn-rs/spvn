use pyo3_build_config::add_extension_module_link_args;

fn main() {
    #[cfg(unix)]
    add_extension_module_link_args();
}