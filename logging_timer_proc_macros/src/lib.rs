#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

// log::LogLevel can be Error, Warn, Info, Debug, Trace.
// Debug is the default if nothing is specified.
// We also allow 'Never' to mean disable timer instrumentation
// altogether. Any casing is allowed.
fn get_log_level_as_string(metadata: proc_macro::TokenStream) -> String {
    const ERR_MSG: &str = "Invalid input: pass a single string literal as the log level, or omit it to default to 'Debug'";

    // println!("**** metadata = {:?}", metadata);
    if metadata.is_empty() {
        return "debug".to_string();
    }

    // We expect 1 string literal.
    let mut i = metadata.into_iter();
    let first_item: proc_macro::TokenTree = i.next().unwrap();

    if i.next().is_some() {
        panic!(ERR_MSG);
    }

    // println!("**** first_item = {:?}", first_item);

    let log_level = match first_item {
        proc_macro::TokenTree::Literal(literal) => literal.to_string(),
        _ => panic!(ERR_MSG),
    };

    // It comes out with an extra set of double quotes!
    let mut log_level = log_level.trim().trim_matches('"').trim().to_string();
    // println!("**** log_level = {:?}", log_level);

    if log_level.is_empty() {
        // If the user specified an empty string, we catch that here.
        log_level += "debug";
    } else {
        // Lowercase simplifies matching later.
        log_level.make_ascii_lowercase();
    }

    log_level
}

/// Instruments the function with a `timer!`, which logs a message at the end of execution
/// including the elapsted time. The attribute accepts a single
/// optional argument to specify the log level. The levels are the same as those used
/// by the `log` crate (error, warn, info, debug and trace) and defaults to "debug".
/// Example:  `#[time("info")]`. You can also specify "never" to
/// completely disable the instrumentation at compile time.
#[proc_macro_attribute]
pub fn time(
    metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut level = get_log_level_as_string(metadata);

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
            _ => panic!("Unrecognized log level: {}", level),
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

// TODO: Get rid of this copy-paste. The only difference is the timer type.


/// Instruments the function with an `stimer!`, which logs a message at the start of execution
/// and at the end including the elapsted time. The attribute accepts a single
/// optional argument to specify the log level. The levels are the same as those used
/// by the `log` crate (error, warn, info, debug and trace) and defaults to "debug".
/// Example:  `#[stime("info")]`. You can also specify "never" to
/// completely disable the instrumentation at compile time.
#[proc_macro_attribute]
pub fn stime(
    metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut level = get_log_level_as_string(metadata);

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
            _ => panic!("Unrecognized log level: {}", level),
        };

        (quote!(
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                let _tmr = stimer!(#log_level; #timer_name);
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
