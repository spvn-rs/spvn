use colored::Colorize;
use proc_macro::TokenStream;

#[proc_macro]
pub fn info(item: TokenStream) -> TokenStream {
    println!("{} {:#?}", "info".blue(), item);
    TokenStream::new()
}
