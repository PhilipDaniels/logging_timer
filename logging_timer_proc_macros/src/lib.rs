#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate darling;
extern crate proc_macro;
use darling::FromMeta;
use log;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    level: Option<String>,
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

    // log::LogLevel can be Error, Warn, Info, Debug, Trace.
    // We also allow 'Never' to mean disable timer instrumentation
    // altogether.
    let mut level = args.level.unwrap_or("debug".to_string());
    if level != "never" {
        let input_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
        let visibility = input_fn.vis;
        let ident = input_fn.ident;
        let inputs = input_fn.decl.inputs;
        let output = input_fn.decl.output;
        let generics = &input_fn.decl.generics;
        let where_clause = &input_fn.decl.generics.where_clause;
        let block = input_fn.block;

        let timer_name = format!("{}()", ident);

        level.make_ascii_lowercase();

        let log_level = match level.as_str() {
            "error" => quote! { log::Level::Error },
            "warn" => quote! { log::Level::Warn },
            "info" => quote! { log::Level::Info  },
            "debug" => quote! { log::Level::Debug  },
            "trace" => quote! { log::Level::Trace  },
            _ => panic!("Unrecognized log level: {}", level)
        };

        (quote!(
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                let _tmr = timer!(#log_level; #timer_name);
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
