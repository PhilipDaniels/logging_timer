#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate darling;
extern crate proc_macro;
use darling::FromMeta;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    print: Option<String>,
    #[darling(default)]
    prefix: Option<String>,
    #[darling(default)]
    suffix: Option<String>,
}

#[proc_macro_attribute]
pub fn stime(
    metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr_args = parse_macro_input!(metadata as syn::AttributeArgs);
    let args: MacroArgs = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let print_arg = args.print.unwrap_or("always".to_string());

    if print_arg.eq(&"always".to_string())
        || (print_arg.eq(&"debug".to_string()) && cfg!(debug_assertions))
    {
        let input_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
        let visibility = input_fn.vis;
        let ident = input_fn.ident;
        let inputs = input_fn.decl.inputs;
        let output = input_fn.decl.output;
        let generics = &input_fn.decl.generics;
        let where_clause = &input_fn.decl.generics.where_clause;
        let block = input_fn.block;
        let mut print_str = "".to_string();
        if let Some(pre) = args.prefix {
            print_str.push_str(&format!("{}::", pre));
        }
        print_str.push_str(&ident.to_string());
        if let Some(suffix) = args.suffix {
            print_str.push_str(&format!("::{}", suffix));
        }

        let timer_name = format!("{}()", ident);

        (quote!(
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                let _tmr = timer!(#timer_name);
                let f = || { #block };
                let r = f();
                r
            }
        ))
        .into()
    } else {
        proc_macro::TokenStream::from(input).into()
    }
}
