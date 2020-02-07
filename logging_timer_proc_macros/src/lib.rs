#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

const DEFAULT_LEVEL: &str = "debug";
const DEFAULT_NAME_PATTERN: &str = "{}";

fn extract_literal(token_tree: &proc_macro::TokenTree) -> String {
    let s = match token_tree {
        proc_macro::TokenTree::Literal(literal) => literal.to_string(),
        _ => panic!("Invalid argument. Specify at most two string literal arguments, for log level and name pattern, in that order.")
    };

    // String literals seem to come through including their double quotes. Trim them off.
    let s = s.trim().trim_matches('"').trim().to_string();
    s
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

    let first_arg = extract_literal(&macro_args[0]);

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
            let mut second_arg = extract_literal(&macro_args[1]);
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

/// Instruments the function with a `timer!`, which logs a message at the end of function
/// execution stating the elapsed time.
///
/// The attribute accepts two string literals as arguments. The first is the log level,
/// valid values of which are "error", "warn", "info", "debug", "trace" or "never".
/// The default value is "debug". "never" can be used to temporarily disable instrumentation
/// of the function without deleting the attribute.
///
/// The second argument is the function name pattern. The pattern is helpful to
/// disambiguate functions when you have many in the same module with the same name: `new`
/// might occur many times on different structs, for example. In the pattern, "{}" will be
/// replaced with the name of the function.
///
/// Examples:
///     #[time]                                 // Use default log level of Debug
///     #[time("info")]                         // Set custom log level
///     #[time("info", "FirstStruct::{}")]      // Logs "FirstStruct::new()" at Info
///     #[time("info", "SecondStruct::{}")]     // Logs "SecondStruct::new()" at Info
///     #[time("ThirdStruct::{}")]              // Logs "ThirdStruct::new()" at Debug
///     #[time("never")]                        // Turn off instrumentation at compile time
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
            "error" => quote! { ::log::Level::Error },
            "warn" => quote! { ::log::Level::Warn },
            "info" => quote! { ::log::Level::Info  },
            "debug" => quote! { ::log::Level::Debug  },
            "trace" => quote! { ::log::Level::Trace  },
            _ => panic!("Unrecognized log level: {}", level),
        };

        (quote!(
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                let _tmr = timer!(#log_level; #timer_name);
                #block
            }
        ))
        .into()
    } else {
        proc_macro::TokenStream::from(input).into()
    }
}

// TODO: Get rid of this copy-paste. The only difference is the timer type.

/// Instruments the function with an `stimer!`, which logs two messages, one at the start
/// of the function and one at the end of execution stating the elapsed time.
///
/// The attribute accepts two string literals as arguments. The first is the log level,
/// valid values of which are "error", "warn", "info", "debug", "trace" or "never".
/// The default value is "debug". "never" can be used to temporarily disable instrumentation
/// of the function without deleting the attribute.
///
/// The second argument is the function name pattern. The pattern is helpful to
/// disambiguate functions when you have many in the same module with the same name: `new`
/// might occur many times on different structs, for example. In the pattern, "{}" will be
/// replaced with the name of the function.
///
/// Examples:
///     #[stime]                                 // Use default log level of Debug
///     #[stime("info")]                         // Set custom log level
///     #[stime("info", "FirstStruct::{}")]      // Logs "FirstStruct::new()" at Info
///     #[stime("info", "SecondStruct::{}")]     // Logs "SecondStruct::new()" at Info
///     #[stime("ThirdStruct::{}")]              // Logs "ThirdStruct::new()" at Debug
///     #[stime("never")]                        // Turn off instrumentation at compile time
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
            "error" => quote! { ::log::Level::Error },
            "warn" => quote! { ::log::Level::Warn },
            "info" => quote! { ::log::Level::Info  },
            "debug" => quote! { ::log::Level::Debug  },
            "trace" => quote! { ::log::Level::Trace  },
            _ => panic!("Unrecognized log level: {}", level),
        };

        (quote!(
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                let _tmr = stimer!(#log_level; #timer_name);
                #block
            }
        ))
        .into()
    } else {
        proc_macro::TokenStream::from(input).into()
    }
}
