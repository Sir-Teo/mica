use std::fmt::Write as _;
use crate::ast::*;

pub fn module_to_string(m: &Module) -> String {
    let mut s = String::new();
    let _ = writeln!(&mut s, "module {}", m.name.join("."));
    for item in &m.items {
        match item {
            Item::Use(u) => {
                let _ = writeln!(&mut s, "use {}{};", u.path.join("."), u.alias.as_ref().map(|a| format!(" as {}", a)).unwrap_or_default());
            }
            Item::TypeAlias(ta) => {
                let _ = write!(&mut s, "{}type {}", if ta.is_public { "pub " } else { "" }, ta.name);
                if !ta.params.is_empty() {
                    let _ = write!(&mut s, "[{}]", ta.params.join(", "));
                }
                let _ = write!(&mut s, " = ");
                fmt_type(&mut s, &ta.value);
                let _ = writeln!(&mut s);
            }
            Item::Function(f) => {
                fmt_fn_sig(&mut s, f);
                let _ = writeln!(&mut s, " {{ … }}");
            }
            Item::Impl(ib) => {
                let _ = write!(&mut s, "impl {} for ", ib.trait_path.segments.join("::"));
                fmt_type(&mut s, &ib.for_type);
                let _ = writeln!(&mut s, " {{ … }}");
            }
        }
    }
    s
}

fn fmt_fn_sig(s: &mut String, f: &Function) {
    let _ = write!(s, "{}fn {}", if f.is_public { "pub " } else { "" }, f.name);
    if !f.generics.is_empty() {
        let mut parts = Vec::new();
        for gp in &f.generics {
            if gp.bounds.is_empty() {
                parts.push(gp.name.clone());
            } else {
                parts.push(format!("{}: {}", gp.name, gp.bounds.iter().map(|p| p.segments.join("::")).collect::<Vec<_>>().join(" + ")));
            }
        }
        let _ = write!(s, "[{}]", parts.join(", "));
    }
    let _ = write!(s, "(");
    let mut first = true;
    for p in &f.params {
        if !first { let _ = write!(s, ", "); } first = false;
        let _ = write!(s, "{}: ", p.name);
        fmt_type(s, &p.ty);
    }
    let _ = write!(s, ")");
    if let Some(ret) = &f.return_type {
        let _ = write!(s, " -> ");
        fmt_type(s, ret);
    }
    if !f.effect_row.is_empty() {
        let _ = write!(s, " !{{{}}}", f.effect_row.join(", "));
    }
}

fn fmt_type(s: &mut String, t: &TypeExpr) {
    match t {
        TypeExpr::Name(n) => { let _ = write!(s, "{}", n); }
        TypeExpr::Generic(n, args) => {
            let _ = write!(s, "{}[", n);
            for (i, a) in args.iter().enumerate() {
                if i > 0 { let _ = write!(s, ", "); }
                fmt_type(s, a);
            }
            let _ = write!(s, "]");
        }
        TypeExpr::Record(fields) => {
            let _ = write!(s, "{{ ");
            for (i, (n, ty)) in fields.iter().enumerate() {
                if i > 0 { let _ = write!(s, ", "); }
                let _ = write!(s, "{}: ", n);
                fmt_type(s, ty);
            }
            let _ = write!(s, " }}");
        }
        TypeExpr::Sum(vars) => {
            for (i, v) in vars.iter().enumerate() {
                if i > 0 { let _ = write!(s, " | "); }
                let _ = write!(s, "{}", v.name);
                if !v.fields.is_empty() {
                    let _ = write!(s, "(");
                    for (j, f) in v.fields.iter().enumerate() {
                        if j > 0 { let _ = write!(s, ", "); }
                        fmt_type(s, f);
                    }
                    let _ = write!(s, ")");
                }
            }
        }
        TypeExpr::List(inner) => { let _ = write!(s, "["); fmt_type(s, inner); let _ = write!(s, "]"); }
        TypeExpr::Tuple(items) => {
            let _ = write!(s, "(");
            for (i, it) in items.iter().enumerate() {
                if i > 0 { let _ = write!(s, ", "); }
                fmt_type(s, it);
            }
            let _ = write!(s, ")");
        }
        TypeExpr::Reference { is_mut, inner } => {
            let _ = write!(s, "&");
            if *is_mut { let _ = write!(s, "mut "); }
            fmt_type(s, inner);
        }
        TypeExpr::Function { params, return_type, effect_row } => {
            let _ = write!(s, "fn(");
            for (i, p) in params.iter().enumerate() {
                if i > 0 { let _ = write!(s, ", "); }
                fmt_type(s, p);
            }
            let _ = write!(s, ") -> ");
            fmt_type(s, return_type);
            if !effect_row.is_empty() {
                let _ = write!(s, " !{{{}}}", effect_row.join(", "));
            }
        }
        TypeExpr::Unit => { let _ = write!(s, "()"); }
    }
}
