#[allow(clippy::E0786)]
use colored::Colorize;

pub fn init_test_hooks() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        {
            eprintln!(
                r#"
                {}
                {:#?}
                "#,
                "⛔️ panic during tests ⛔️ ".red().bold(),
                info,
            );
        }
        hook(info);
        std::process::exit(1);
    }));
}
