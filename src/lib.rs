use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{ItemFn, FieldValue, Member, Expr, Lit};

#[proc_macro_attribute]
pub fn test_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_ast: ItemFn = syn::parse(item).expect("Expected function definition");
    let attr_ast: FieldValue = syn::parse(attr).expect("Expected description of dataset");

    impl_test_data(&attr_ast, &item_ast)
}

fn impl_test_data(attr_ast: &FieldValue, item_ast: &ItemFn) -> TokenStream {
    let mut test_count = match &attr_ast.expr {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(n) => n.base10_parse::<usize>().unwrap(),
            _ => panic!("Number of tests not number!"),
        }
        _ => panic!("Number of tests not number!"),
    };
    let data_name = match &attr_ast.member {
        Member::Named(id) => id,
        _ => panic!("Expected name of dataset"),
    };
    let proxy_name = &item_ast.sig.ident;
    let arguments_count = item_ast.sig.inputs.len();
    let test_name = "test";
    let mut gen = String::new();
    let mut gen_cur;
    let mut name_cur;
    let mut call;
    let mut arguments;
    let mut i;
    while test_count > 0 {
        test_count -= 1;

        i = 1;
        arguments = format!("{}[{}].0[0]", data_name, test_count);
        while i < arguments_count {
            arguments = format!("{}, {}[{}].0[{}]", arguments, data_name, test_count, i);
            i += 1;
        }
        name_cur = format_ident!("{}_{}", test_name, test_count);
        call = format!("{}({})", proxy_name, arguments);
        gen_cur = format!("#[test]\nfn {}() {{assert!({} == {}[{}].1)}}", name_cur, call, data_name, test_count);
        gen = format!("{}{}", gen, gen_cur);
    }
    let mut res = gen.parse().expect("Should been converted");
    item_ast.to_tokens(&mut res);
    res.into()
}
