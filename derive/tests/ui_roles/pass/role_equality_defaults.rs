use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::{Entity, Enum, Model, Projection, Value};

#[Entity(id = "trybuild.FloatEntity")]
struct FloatEntity {
    #[identifier]
    id: Id,
    amount: f64,
}

#[Projection(source = FloatEntity)]
struct FloatProjection {
    #[identifier]
    id: Id,
    amount: f64,
}

#[Model]
struct FloatModel { amount: f64 }

#[Entity(id = "trybuild.HashEntity", eq, hash)]
struct HashEntity {
    #[identifier]
    id: Id,
}

#[Projection(source = HashEntity, eq, hash)]
struct HashProjection {
    #[identifier]
    id: Id,
}

#[Model(eq, hash)]
struct HashModel { value: u8 }

#[Value]
struct Scalar(u8);

#[Enum]
enum Choice { One }

#[Model(ord)]
struct Ordered { value: u8 }

// A handwritten Hash impl verifies that ord does not generate Hash.
impl std::hash::Hash for Ordered {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.value, state);
    }
}

#[Model(eq)]
struct EqualityOnly { value: u8 }

// Likewise, eq alone must not opt into hashing.
impl std::hash::Hash for EqualityOnly {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.value, state);
    }
}

#[Model(ord, hash)]
struct OrderedHash { value: u8 }

#[Model(partial_ord)]
struct PartialOrder { value: f64 }

fn partial<T: PartialEq>() {}
fn hashed<T: Eq + std::hash::Hash>() {}
fn ordered<T: Eq + Ord>() {}
fn partially_ordered<T: PartialOrd>() {}

fn main() {
    partial::<FloatEntity>();
    partial::<FloatProjection>();
    partial::<FloatModel>();
    partially_ordered::<PartialOrder>();
    hashed::<HashEntity>();
    hashed::<HashProjection>();
    hashed::<HashModel>();
    hashed::<Scalar>();
    hashed::<Choice>();
    ordered::<Ordered>();
    hashed::<EqualityOnly>();
    hashed::<OrderedHash>();
}
