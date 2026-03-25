#[cfg(test)]
mod tests;

mod evalstate;
mod evalstatebuilder;
mod realised_string;
mod value;
mod values;
mod valuetype;

pub use evalstate::EvalState;
pub use evalstatebuilder::EvalStateBuilder;
pub use realised_string::RealisedString;
pub use value::Value;
pub use values::{NixInt, NixValue};
pub use valuetype::ValueType;
