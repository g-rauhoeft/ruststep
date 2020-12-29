use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    // simple types
    Number,
    Real,
    Integer,
    String,
    Boolean,
    Logical,
    Binary(Option<usize>),

    // aggregate types
    Array {
        index_range: (isize, isize),
        optional: bool,
        unique: bool,
        base_type: String,
    },
    List {
        length_range: (usize, Option<usize>),
        unique: bool,
        base_type: String,
    },
    Bag {
        length_range: (usize, Option<usize>),
        base_type: String,
    },
    Set {
        length_range: (usize, Option<usize>),
        base_type: String,
    },
    // named types
    Entity {
        name: String,
        members: Vec<MemberVariant>,
    },
    Defined {
        name: String,
        underlying: String,
    },

    // constructed types
    Enumerate {
        name: String,
        items: Vec<String>,
    },
    Select {
        name: String,
        items: Vec<String>,
    },

    // generalized types
    Generic,
    Aggrigate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberVariant {
    pub name: String,
    pub type_name: String,
    pub optional: bool,
}

impl Type {
    pub fn is_simple(&self) -> bool {
        match self {
            Type::Number => true,
            Type::Real => true,
            Type::Integer => true,
            Type::String => true,
            Type::Boolean => true,
            Type::Logical => true,
            Type::Binary(_) => true,
            _ => false,
        }
    }

    pub fn is_aggrigation(&self) -> bool {
        match self {
            Type::Array { .. } => true,
            Type::List { .. } => true,
            Type::Bag { .. } => true,
            Type::Set { .. } => true,
            _ => false,
        }
    }

    pub fn is_named(&self) -> bool {
        match self {
            Type::Entity { .. } => true,
            Type::Defined { .. } => true,
            _ => false,
        }
    }

    pub fn is_constructed(&self) -> bool {
        match self {
            Type::Enumerate { .. } => true,
            Type::Select { .. } => true,
            _ => false,
        }
    }

    pub fn is_generalized(&self) -> bool {
        match self {
            Type::Generic => true,
            Type::Aggrigate => true,
            _ => false,
        }
    }

    pub fn is_base(&self) -> bool {
        self.is_aggrigation() | self.is_simple() | self.is_named()
    }

    pub fn is_parameter(&self) -> bool {
        self.is_generalized() | self.is_named() | self.is_simple()
    }

    pub fn is_underlying(&self) -> bool {
        match self {
            Type::Defined { .. } => true,
            _ => self.is_constructed() | self.is_aggrigation() | self.is_simple(),
        }
    }

    pub fn rust_type_name(&self) -> String {
        match self {
            Type::Number => "f64".into(),
            Type::Real => "f64".into(),
            Type::Integer => "isize".into(),
            Type::String => "String".into(),
            Type::Boolean => "bool".into(),
            Type::Logical => "Logical".into(),
            Type::Binary(None) => "Vec<bool>".into(),
            Type::Binary(Some(len)) => format!("[bool; {}]", len),
            Type::Array {
                index_range,
                base_type,
                ..
            } => format!("[{}; {}]", base_type, index_range.1 - index_range.0),
            Type::List { base_type, .. } => format!("Vec<{}>", base_type),
            Type::Bag { base_type, .. } => format!("Vec<{}>", base_type),
            Type::Set { base_type, .. } => format!("HashSet<{}>", base_type),
            Type::Entity { name, .. } => name.to_pascal_case(),
            Type::Defined { name, .. } => name.to_pascal_case(),
            Type::Enumerate { name, .. } => name.to_pascal_case(),
            Type::Select { name, .. } => name.to_pascal_case(),
            Type::Generic => "dyn Any".into(),
            Type::Aggrigate => "dyn Any".into(),
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            Type::Entity { members, .. } => {
                entity_struct_definition(&self.rust_type_name(), members)
            }
            Type::Defined { underlying, .. } => {
                defined_type_struct(&self.rust_type_name(), &underlying)
            }
            _ => unimplemented!(),
        })
    }
}

fn entity_struct_definition(name: &str, members: &[MemberVariant]) -> TokenStream {
    let name = format_ident!("{}", name);
    let member_name: Vec<_> = members
        .iter()
        .map(|member| format_ident!("{}", member.name))
        .collect();
    let member_type: Vec<_> = members
        .iter()
        .map(|member| {
            let name = format_ident!("{}", member.type_name.to_pascal_case());
            if member.optional {
                quote! { Option<#name> }
            } else {
                quote! { #name }
            }
        })
        .collect();

    quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #name {
            #(
            #member_name : #member_type,
            )*
        }

        impl #name {
            pub fn new(#(#member_name : #member_type),*) -> Self {
                Self { #(#member_name),* }
            }
        }
    }
}

fn defined_type_struct(name: &str, underlying: &str) -> TokenStream {
    let name = format_ident!("{}", name);
    let underlying = format_ident!("{}", underlying);

    quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #name ( #underlying );

        impl #name {
            pub fn new(entity: #underlying) -> Self {
                Self ( entity )
            }
        }
    }
}

/// Corresponding to `SCHEMA` in EXPRESS
///
/// Here, we consinder following simple schema definition in EXPRESS language
///
/// ```text
/// SCHEMA ONE;
///
/// ENTITY first;
/// m_ref : second;
/// fattr : STRING;
/// END_ENTITY;
///
/// ENTITY second;
/// sattr : STRING;
/// END_ENTITY;
///
/// END_SCHEMA;
/// ```
///
/// EXPRESS's schema consists of `ENTITY`es,
/// which will be translated into Rust struct definitions.
///
#[derive(Clone, Debug)]
pub struct Schema {
    pub name: String,
    pub types: Vec<Type>,
}

impl ToTokens for Schema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name);
        let types = &self.types;
        tokens.append_all(quote! {
            mod #name {
                #(#types)*
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_entity_definition() {
        let entity = Type::Entity {
            name: "test_struct_type".into(),
            members: vec![
                MemberVariant {
                    name: "m_int".into(),
                    type_name: "usize".into(),
                    optional: true,
                },
                MemberVariant {
                    name: "m_float".into(),
                    type_name: "f64".into(),
                    optional: false,
                },
            ],
        };
        println!("{}", entity.to_token_stream());
    }

    #[test]
    fn print_defined_definition() {
        let defined = Type::Defined {
            name: "test_defined_type".into(),
            underlying: "FormerType".into(),
        };
        println!("{}", defined.to_token_stream());
    }
}