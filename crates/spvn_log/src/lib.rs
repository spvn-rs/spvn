
use proc_macro::TokenStream;
use colored::Colorize;


#[proc_macro]
pub fn info(item: TokenStream)  -> TokenStream {
    println!("{} {:#?}", "info".blue(), item);
    TokenStream::new()
}