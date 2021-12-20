extern crate proc_macro;

use proc_macro::TokenStream;

mod common;

mod command;
mod commands;
mod group;

#[proc_macro_derive(Command, attributes(command, option))]
pub fn derive_command(item: TokenStream) -> TokenStream {
    command::derive_command(item.into()).unwrap_or_else(|e| e.into_compile_error()).into()
}

#[proc_macro_derive(Commands)]
pub fn derive_commands(item: TokenStream) -> TokenStream {
    commands::derive_commands(item.into()).unwrap_or_else(|e| e.into_compile_error()).into()
}

#[proc_macro_derive(Group, attributes(group, option))]
pub fn derive_group(item: TokenStream) -> TokenStream {
    group::derive_group(item.into()).unwrap_or_else(|e| e.into_compile_error()).into()
}
