//! Traits for espr-generated structures
//!
//! Examples
//! ---------
//!
//! Convert STEP data section into tables.
//!
//! ```
//! use ruststep::{parser::exchange, ap000::Ap000};
//! use nom::Finish;
//!
//! let (_, data_section) = exchange::data_section(
//!     r#"
//!     DATA;
//!       #2 = A(1.0, 2.0);
//!       #4 = B(2.0, A((4.0, 5.0)));
//!       #5 = B(2.0, #2);
//!     ENDSEC;
//!     "#
//!     .trim(),
//! )
//! .finish()
//! .unwrap();
//!
//! // Entity reference `#2` is not resolved at this point.
//! let table = Ap000::from_section(&data_section).unwrap();
//!
//! for a in table.a_iter() { // Iterate over top-level `A`s.
//!   dbg!(a);                // Do not iterate over the inline struct `A((4.0, 5.0))` in `#4`
//! }
//!
//! for b in table.b_iter() { // Because reference lookup is done while iteration,
//!   dbg!(b);                // `b` may be `Result::Err` if reference is undefined.
//! }
//!
//! for c in table.c_iter() { // No iteration occurs since `C` is not defined
//!   dbg!(c);
//! }
//! ```
//!
//! STEP exchange structure AST is converted into Rust structure in two steps:
//!
//! 1.  Parse AST to `*Holder` private struct without resolving entity references,
//!    e.g. `B(2.0, #2)` will be converted into a Rust struct
//!         `BHolder { z: 2.0, a: RValue::Entity(2)}`.
//!     - `Ap000::from_section` as above example
//! 2. Resolve reference `#2` to `A(1.0, 2.0)`.
//!    This will be done while the iteration.
//!     - `a_iter()` as above example
//!

use crate::ast::*;
use serde::{
    de::{self, IntoDeserializer, VariantAccess},
    Deserialize,
};
use std::{collections::HashMap, fmt, marker::PhantomData};

#[cfg(doc)]
use crate::ap000;

/// Trait for resolving a reference through entity id
///
/// STEP input is once deserialized into a holder struct, e.g. [ap000::AHolder],
/// which contains [PlaceHolder] for skipping table lookup.
/// Entity table represented by the trait [EntityTable] will be looked up to resolve the reference
/// at [PlaceHolder::into_owned], and this trait is for supporting this method.
///
/// Holder structs, e.g. [ap000::AHolder] will be generated by espr,
/// and implementation of this trait will be also generated at the same time.
///
pub trait Holder {
    type Owned;
    type Table;
    fn into_owned(self, table: &Self::Table) -> Result<Self::Owned, crate::error::Error>;
}

/// Trait for tables which pulls an entity (`T`) from an entity id (`u64`)
pub trait EntityTable<T> {
    fn get_entity(&self, entity_id: u64) -> Result<&T, crate::error::Error>;
}

impl<T> EntityTable<T> for HashMap<u64, T> {
    fn get_entity(&self, id: u64) -> Result<&T, crate::error::Error> {
        Ok(self
            .get(&id)
            .ok_or_else(|| crate::error::Error::UnknownEntity(id))?)
    }
}

/// Owned value or reference through entity/value id
#[derive(Debug, Clone, PartialEq)]
pub enum PlaceHolder<T> {
    Ref(RValue),
    Owned(T),
}

impl<T: Holder> PlaceHolder<T> {
    /// Get owned value, or look up entity table and clone it for a reference.
    ///
    /// Errors
    /// -------
    /// - if table lookup failed, i.e. unknown entity id not registered in the table
    ///
    pub fn into_owned<Table>(self, table: &Table) -> Result<T::Owned, crate::error::Error>
    where
        T: Holder<Table = Table> + Clone,
        Table: EntityTable<T>,
    {
        let value = match self {
            PlaceHolder::Ref(id) => match id {
                RValue::Entity(id) => table.get_entity(id)?.clone(),
                _ => unimplemented!("ENTITY is only supported now"),
            },
            PlaceHolder::Owned(a) => a,
        };
        Ok(value.into_owned(table)?)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for PlaceHolder<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // Dispatched dynamically:
        //
        // For Ref(RValue)
        // ----------------
        // PlaceHolder::deserialize(RValue)
        // > RValue::deserialize_struct(PlaceHolderVisitor)
        // > (forward_to_deserialize_any)
        // > RValue::deserialize_any(PlaceHolderVisitor)
        // > PlaceHolderVisitor::visit_enum(MapAccessDeserializer)
        //
        // For Owned(T)
        // -------------
        // PlaceHolder::deserialize(Record)
        // > Record::deserialize_struct(PlaceHolderVisitor)
        // > PlaceHolderVisitor::visit_seq(SeqDeserializer)
        deserializer.deserialize_struct(
            std::any::type_name::<T>(),
            &[],
            PlaceHolderVisitor::<T> {
                phantom: PhantomData,
            },
        )
    }
}

struct PlaceHolderVisitor<T> {
    phantom: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> de::Visitor<'de> for PlaceHolderVisitor<T> {
    type Value = PlaceHolder<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "PlaceHolder<{}>", std::any::type_name::<T>())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PlaceHolder::Owned(T::deserialize(v.into_deserializer())?))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PlaceHolder::Owned(T::deserialize(v.into_deserializer())?))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PlaceHolder::Owned(T::deserialize(v.into_deserializer())?))
    }

    // For Ref(RValue)
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let (key, variant): (String, _) = data.variant()?;
        match key.as_str() {
            "Entity" => {
                let value: u64 = variant.newtype_variant()?;
                Ok(PlaceHolder::Ref(RValue::Entity(value)))
            }
            "Value" => {
                let value: u64 = variant.newtype_variant()?;
                Ok(PlaceHolder::Ref(RValue::Value(value)))
            }
            "ConstantEntity" => {
                let name: String = variant.newtype_variant()?;
                Ok(PlaceHolder::Ref(RValue::ConstantEntity(name)))
            }
            "ConstantValue" => {
                let name: String = variant.newtype_variant()?;
                Ok(PlaceHolder::Ref(RValue::ConstantValue(name)))
            }
            _ => unreachable!("Invalid key while deserializing PlaceHolder"),
        }
    }

    // For Owned(T)
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut components: Vec<Parameter> = Vec::new();
        while let Some(component) = seq.next_element()? {
            components.push(component);
        }
        let seq = de::value::SeqDeserializer::new(components.iter());
        Ok(PlaceHolder::Owned(T::deserialize(seq).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::exchange;
    use nom::Finish;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct A {
        x: f64,
        y: f64,
    }

    #[test]
    fn place_holder() {
        let value = RValue::Entity(11);
        let a: PlaceHolder<A> = Deserialize::deserialize(&value).unwrap();
        dbg!(a);

        let value = RValue::ConstantValue("VIM".into());
        let a: PlaceHolder<A> = Deserialize::deserialize(&value).unwrap();
        dbg!(a);

        let (_, record) = exchange::simple_record("A(1.0, 2.0)").finish().unwrap();
        let a: PlaceHolder<A> = Deserialize::deserialize(&record).unwrap();
        dbg!(a);
    }
}