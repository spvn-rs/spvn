use spvn_caller::resolve_import;

fn main() {
    let import_result = resolve_import("pylib.foo:app");
    let caller = match import_result {
        Ok(asgi_app) => asgi_app,
        Err(_) => return Result::Ok(ExitStatus::Error),
    };
}
