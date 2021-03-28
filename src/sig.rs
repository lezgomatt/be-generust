use syn::spanned::Spanned;

type IterParams = (proc_macro2::Ident, syn::Type);
type IterSig = (Vec<IterParams>, syn::Type);

macro_rules! forbid_some {
    ($e:expr, $msg:expr) => {
        if $e.is_some() {
            return Err((&$e, $msg));
        }
    }
}

pub(crate) fn extract_iter_sig(sig: &syn::Signature) -> Result<IterSig, (&dyn Spanned, &str)> {
    forbid_some!(sig.constness, "iterator cannot be const");
    forbid_some!(sig.asyncness, "iterator cannot be async");
    forbid_some!(sig.unsafety, "iterator cannot be unsafe");
    forbid_some!(sig.abi, "iterator cannot be extern");
    forbid_some!(sig.variadic, "iterator cannot have variadic parameters");

    if let Some(arg) = sig.inputs.first() {
        if let syn::FnArg::Receiver(_) = arg {
            return Err((arg, "iterator cannot have a method receiver (self)"));
        }
    }

    let mut params = Vec::<(proc_macro2::Ident, syn::Type)>::new();

    for arg in sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => {
                    if pat_ident.by_ref.is_some() {
                        return Err((
                            &pat_ident.by_ref,
                            "iterator cannot have reference arguments",
                        ));
                    }

                    if let Some((_, ref subpat)) = pat_ident.subpat {
                        return Err((subpat, "iterator cannot have argument subpatterns"));
                    }

                    params.push((pat_ident.ident.clone(), (*pat_type.ty).clone()));
                }
                _ => {
                    return Err((arg, "iterator cannot have a pattern arguments"));
                }
            }
        }
    }

    match get_iter_item_type(&sig.output) {
        Some(ty) => Ok((params, ty.clone())),
        None => {
            return Err((
                &sig.output,
                "return type must be `-> impl Iterator<Item = XXX>`",
            ));
        }
    }
}

macro_rules! must_match {
    ($e:expr, $p:pat, $r:expr) => {
        match $e {
            $p => $r,
            _ => return None,
        }
    };
}

macro_rules! sole_elem {
    ($e:expr) => {
        if $e.len() == 1 {
            &$e[0]
        } else {
            return None;
        }
    };
}

fn get_iter_item_type(ret_type: &syn::ReturnType) -> Option<&syn::Type> {
    let boxed_type = must_match!(ret_type, syn::ReturnType::Type(_, bt), bt);

    // impl Iterator<Item = XXX>
    let impl_bound = must_match!(
        **boxed_type,
        syn::Type::ImplTrait(ref it),
        sole_elem!(it.bounds)
    );

    // Iterator<Item = XXX>
    let trait_segment = must_match!(
        impl_bound,
        syn::TypeParamBound::Trait(tb),
        sole_elem!(tb.path.segments)
    );
    if trait_segment.ident.to_string() != "Iterator" {
        return None;
    }

    // <Item = XXX>
    let generic_arg = must_match!(
        trait_segment.arguments,
        syn::PathArguments::AngleBracketed(ref ga),
        sole_elem!(ga.args)
    );

    // Item = XXX
    let binding = must_match!(generic_arg, syn::GenericArgument::Binding(ref b), b);
    if binding.ident.to_string() != "Item" {
        return None;
    }

    return Some(&binding.ty);
}
