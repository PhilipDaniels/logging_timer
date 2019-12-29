#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

const DEFAULT_LEVEL: &str = "debug";
const DEFAULT_NAME_PATTERN: &str = "{}";

fn extract_literal(token_tree: &proc_macro::TokenTree) -> String {
    match token_tree {
        proc_macro::TokenTree::Literal(literal) => literal.to_string(),
        _ => panic!("Invalid argument. Specify at most two string literal arguments, for log level and name pattern, in that order.")
    }
}

// log::LogLevel can be Error, Warn, Info, Debug, Trace.
// Debug is the default if nothing is specified.
// We also allow 'Never' to mean disable timer instrumentation
// altogether. Any casing is allowed.
fn get_log_level_and_name_pattern(metadata: proc_macro::TokenStream) -> (String, String) {
    // Grab everything into a vector and filter out any intervening punctuation
    // (commas come through as TokenTree::Punct(_)).
    let macro_args: Vec<proc_macro::TokenTree> = metadata
        .into_iter()
        .filter(|token| match token {
            proc_macro::TokenTree::Literal(_) => true,
            _ => false,
        })
        .collect();
    //println!("macro_args = {:#?}", macro_args);

    if macro_args.is_empty() {
        return (DEFAULT_LEVEL.to_string(), DEFAULT_NAME_PATTERN.to_string());
    }

    if macro_args.len() > 2 {
        panic!("Specify at most two string literal arguments, for log level and name pattern");
    }

    let first_arg = match &macro_args[0] {
        proc_macro::TokenTree::Literal(literal) => literal.to_string(),
        _ => panic!("Invalid first argument. Specify at most two string literal arguments, for log level and name pattern."),
    };

    // String literals seem to come through including their double quotes. Trim them off.
    let first_arg = first_arg.trim().trim_matches('"').trim();

    if first_arg.contains("{}") && macro_args.len() == 2 {
        panic!("Invalid first argument. Specify the log level as the first argument and the pattern as the second.");
    }

    let first_arg_lower = first_arg.to_ascii_lowercase();
    if macro_args.len() == 1 {
        match first_arg_lower.as_str() {
            "error" | "warn" | "info" | "debug" | "trace" | "never" => {
                // User specified a valid log level as their only argument.
                return (first_arg_lower, DEFAULT_NAME_PATTERN.to_string())
            }
            _ => {
                // User specified something that doesn't look like a log-level.
                // It may be a pattern with "{}", or it may just be a string.
                // In any case, consider it to be the pattern and return it
                // n.b. the original, not the lowered version.
                return (DEFAULT_LEVEL.to_string(), first_arg.to_string())
            },
        }
    }

    // We have two arguments. We are stricter on the first now, we require
    // that to be a valid log level.
    match first_arg_lower.as_str() {
        "error" | "warn" | "info" | "debug" | "trace" | "never" => {
            let second_arg = match &macro_args[1] {
                proc_macro::TokenTree::Literal(literal) => literal.to_string(),
                _ => panic!("Invalid second argument. Specify at most two string literal arguments, for log level and name pattern.")
            };
            let mut second_arg = second_arg.trim().trim_matches('"').trim().to_string();
            if second_arg.is_empty() {
                second_arg += DEFAULT_NAME_PATTERN;
            }

            return (first_arg_lower, second_arg.to_string())
        }
        _ => panic!("Invalid first argument. Specify the log level as the first argument and the pattern as the second.")
    }
}

fn get_timer_name(name_pattern: &str, fn_name: &str) -> String {
    let fn_name_with_parens = format!("{}()", fn_name);
    let timer_name = name_pattern.replacen("{}", &fn_name_with_parens, 1);
    timer_name
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
    let (level, name_pattern) = get_log_level_and_name_pattern(metadata);

    if level != "never" {
        let input_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
        let visibility = input_fn.vis;
        let ident = input_fn.ident;
        let inputs = input_fn.decl.inputs;
        let output = input_fn.decl.output;
        let generics = &input_fn.decl.generics;
        let where_clause = &input_fn.decl.generics.where_clause;
        let block = input_fn.block;

        let timer_name = get_timer_name(&name_pattern, &ident.to_string());

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
    let (level, name_pattern) = get_log_level_and_name_pattern(metadata);

    if level != "never" {
        let input_fn: syn::ItemFn = parse_macro_input!(input as syn::ItemFn);
        let visibility = input_fn.vis;
        let ident = input_fn.ident;
        let inputs = input_fn.decl.inputs;
        let output = input_fn.decl.output;
        let generics = &input_fn.decl.generics;
        let where_clause = &input_fn.decl.generics.where_clause;
        let block = input_fn.block;

        let timer_name = get_timer_name(&name_pattern, &ident.to_string());

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
