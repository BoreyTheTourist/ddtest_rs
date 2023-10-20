//! Data Driven Tests for Rust
//!
//! Provides macro attribute that makes it possible to write tests in Data Driven style([`git`])
//!
//! [`git`]: https://github.com/BoreyTheTourist/ddtest_rs
use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{ItemFn, FieldValue, Member, Expr, Lit};

/// The whole point
///
/// # Usage
/// This macro works properly only with parameter of folowing format:
/// ```text
/// {name}: {number_of_tests}
/// ```
///
/// where
/// - {_name_} is the name of variable/const that contains dataset and 
/// - {_number_of_tests_} - number of test cases, or, what's the same, length of dataset
///
/// Target item of this macro is function, that asserts sth using it's inputs, like following
/// ```
/// fn add_test(x: i32, y: i32, res: i32) { 
///     assert!(x + y == res)
/// }
/// ```
///
/// And so, there's certain demands to format of dataset. Every single test case is tuple of
/// input parameters. As example, consider you want to test `add` function. Test dataset for such case: 
/// ```
/// const DATA: [(i32, i32, i32); 2] = [
///     ( 2, 3,   5 ),
///     ( -1, 4,  3 ),
/// ];
/// ```
///
/// That's, full example:
///
/// ```
/// # use ddtest_rs::test_data;
/// const DATA: [(i32, i32, i32); 2] = [
///     ( 2, 3,   5 ),
///     ( -1, 4,  3 ),
/// ];
///
/// #[test_data(DATA: 2)]
/// fn add_test(x: i32, y: i32, res: i32) {
///     assert!(x + y == res)
/// }
/// ```
///
/// This will produce following code(that will be placed __inside of scope__ of `add` function):
/// ```
/// #[test]
/// fn test_0() {
///     add(2, 3, 5)
/// }
///
/// #[test]
/// fn test_1() {
///     add(-1, 4, 3)
/// }
/// ```
///
/// # Panics
///
/// - Target item is not function definition
/// ```compile_fail
/// #[test_data(DATA: 1)]
/// struct SomeStruct;
/// ```
///
/// - Wrong attribute format
/// ```compile_fail
/// #[test_data(DATA)]
/// fn add_test(x: i32, y: i32, res: i32) {
///     assert!(x + y == res)
/// }
/// ```
///
/// - Not existing dataset
///
/// ```compile_fail
/// const DATA: [(i32, i32, i32); 2] = [
///     ( 2, 3,   5 ),
///     ( -1, 4,  3 ),
/// ];
///
/// #[test_data(DATA1: 2)]
/// fn add_test(x: i32, y: i32, res: i32) {
///     assert!(x + y == res)
/// }
/// ```
///
/// Also, test will fail if number of test cases more than actual len of dataset:
/// ```
/// # use ddtest_rs::test_data;
/// const DATA: [(i32, i32, i32); 2] = [
///     ( 2, 3,   5 ),
///     ( -1, 4,  3 ),
/// ];
///
/// #[test_data(DATA: 3)]
/// fn add_test(x: i32, y: i32, res: i32) {
///     assert!(x + y == res)
/// }
/// ```
/// There is, `test_2` panics.
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
    let mut gen = String::new();
    let mut gen_cur;
    let mut name_cur;
    let mut call;
    let mut arguments;
    let mut i;
    while test_count > 0 {
        test_count -= 1;

        i = 1;
        arguments = match arguments_count == 1 {
            true => format!("{}[{}]", data_name, test_count),
            _ => format!("{}[{}].0", data_name, test_count),
        };
        while i < arguments_count {
            arguments = format!("{}, {}[{}].{}", arguments, data_name, test_count, i);
            i += 1;
        }
        name_cur = format_ident!("{}_{}", proxy_name, test_count);
        call = format!("{}({})", proxy_name, arguments);
        gen_cur = format!("#[test]\nfn {}() {{{}}}", name_cur, call);
        gen = format!("{}{}", gen, gen_cur);
    }
    let mut res = gen.parse().expect("Should been converted");
    item_ast.to_tokens(&mut res);
    res.into()
}
