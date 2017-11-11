extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

const DISPLAY_ATTR: &'static str = "display";
const DISPLAY_MSG: &'static str = "msg";
const CAUSE_ATTR: &'static str = "cause";

decl_derive!([YadeError, attributes(display, cause)] => error_derive);
fn error_derive(s: synstructure::Structure) -> quote::Tokens {
    let display_impl = generate_display_impl(&s);
    let error_impl = generate_error_impl(&s);

    quote! {
        #display_impl
        #error_impl
    }
}

decl_derive!([YadeKind, attributes(display)] => error_kind_derive);
fn error_kind_derive(s: synstructure::Structure) -> quote::Tokens {
    generate_display_impl(&s)
}

const INVALID_CAUSE: &'static str = "Cause must be a type implementing Error, a Box<Error>, or an Option<Box<Error>>";

fn generate_error_impl(s: &synstructure::Structure) -> quote::Tokens {
    let cause_matches = s.each_variant(|v| {
        if let Some(cause) = v.bindings().iter().find(is_cause) {
            let path = match cause.ast().ty {
                syn::Ty::Path(_, ref path) => path,
                _ => panic!(INVALID_CAUSE),
            };
            let parent_type = path.segments.last().unwrap();

            match parent_type.ident.as_ref() {
                "Option" => {
                    let child_path = match parent_type.parameters {
                        syn::PathParameters::AngleBracketed(ref data) => {
                            match data.types[0] {
                                syn::Ty::Path(_, ref path) => path,
                                _ => panic!(INVALID_CAUSE),
                            }
                        },
                        _ => panic!(INVALID_CAUSE),
                    };
                    let child_type = child_path.segments.last().unwrap();
                    match child_type.ident.as_ref() {
                        "Box" => {
                            quote!(return #cause.as_ref().map(|c|c.as_ref()))
                        },
                        _ => panic!(INVALID_CAUSE),
                    }

                },
                "Box" => quote!(return Some(#cause.as_ref())),
                _ => quote!(return Some(#cause)),
            }
        } else {
            quote!(return None)
        }
    });

    s.bound_impl("::std::error::Error", quote! {
        #[allow(unreachable_code)]
        fn cause(&self) -> Option<&::std::error::Error> {
            match *self { #cause_matches }
            None
        }

        #[allow(unreachable_code)]
        fn description(&self) -> &str {
            "For a description please use the Display trait implementation of this Error"
        }
    })
}

fn generate_display_impl(s: &synstructure::Structure) -> quote::Tokens {
    let display_matches = s.each_variant(|v| {
        let ast = &v.ast();
        let name = ast.ident.to_string();

        let msg = match find_error_msg(&ast.attrs) {
            Some(msg) => msg,
            None => {
                return quote! {
                    return write!(f, "{}", #name);
                }
            }
        };

        if msg.is_empty() {
            panic!("Attribute '{}' must contain `{} = \"\"`", DISPLAY_ATTR, DISPLAY_MSG);
        }

        let s = match msg[0] {
            syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref i, ref lit)) if i == DISPLAY_MSG => {
                lit.clone()
            }
            _ => panic!("Attribute '{}' must contain `{} = \"\"`", DISPLAY_ATTR, DISPLAY_MSG),
        };
        use quote::ToTokens;
        let args = msg[1..].iter().map(|arg| match *arg {
            syn::NestedMetaItem::Literal(syn::Lit::Int(i, _)) => {
                let bi = &v.bindings()[i as usize];
                quote!(#bi)
            }
            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref id)) => {
                if id.as_ref().starts_with("_") {
                    if let Ok(idx) = id.as_ref()[1..].parse::<usize>() {
                        let bi = &v.bindings()[idx];
                        return quote!(#bi)
                    }
                }
                for bi in v.bindings() {
                    if bi.ast().ident.as_ref() == Some(id) {
                        return quote!(#bi);
                    }
                }
                panic!("Couldn't find field '{}' of '{}'", id, name);
            }
            _ => {
                let mut tokens = quote::Tokens::new();
                arg.to_tokens(&mut tokens);
                panic!("Invalid '{}' attribute argument `{}` for '{}'", DISPLAY_ATTR, tokens, name);
            },
        });

        quote! {
            return write!(f, #s #(, #args)*)
        }
    });

    s.bound_impl("::std::fmt::Display", quote! {
        #[allow(unreachable_code)]
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self { #display_matches }
            write!(f, "There was an unknown error.")
        }
    })
}

fn find_error_msg(attrs: &[syn::Attribute]) -> Option<&[syn::NestedMetaItem]> {
    let mut error_msg = None;

    for attr in attrs {
        if attr.name() == DISPLAY_ATTR {
            if error_msg.is_some() {
                panic!("Attribute '{}' specified multiple times.", DISPLAY_ATTR)
            } else {
                if let syn::MetaItem::List(_, ref list)  = attr.value {
                    error_msg = Some(&list[..]);
                } else {
                    panic!("Attribute '{}' requires parens", DISPLAY_ATTR)
                }
            }
        }
    }

    error_msg
}

fn is_cause(bi: &&synstructure::BindingInfo) -> bool {
    bi.ast().attrs.iter().any(|attr| attr.name() == CAUSE_ATTR)
}
