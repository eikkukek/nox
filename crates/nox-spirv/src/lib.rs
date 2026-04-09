//! Compact SPIR-V reflection library written in pure-Rust, with zero dependencies and minimal
//! allocations.
//!
//! # Allocation policy
//! - Strings and arrays are just slices to the SPIR-V passed to [`Module`].
//! - Only [`decorations`][1] and [`instructions streams`][2] with a [`result id`][3] are cached using
//!   an extra allocation.
//! - Parsing is mostly done on-demand.
//! 
//! [1]: op::Decoration
//! [2]: stream::InstructionStream
//! [3]: op::IdResult
//! # Usage 
//! ``` rust
//! use nox_spirv::op;
//! use nox_spirv::Module;
//! use nox_spirv::reflect::{Reflector, ResourceType};
//! 
//! let spirv: &[u32] = ...;
//! let module = Module::new(spirv);
//! let mut reflector = Reflector::new(module).unwrap();
//! // Must be set before reflecting resources.
//! reflector.set_entry_point(c"main", op::ExecutionModel::FRAGMENT).unwrap();
//! for ubo in reflector.resources_for_type(ResourceType::UniformBuffer).unwrap() {
//!     match ubo {
//!         Ok(ubo) => {
//!             let mut set = None;
//!             let mut binding = None;
//!             for dec in reflector.decorations(ubo.variable_id) {
//!                 if let op::Decoration::DescriptorSet { descriptor_set } = dec.decoration {
//!                     set = Some(descriptor_set);
//!                 } else if let op::Decoration::Binding { binding_point } = dec.decoration {
//!                     binding = Some(binding_point);
//!                 }
//!             }
//!             println!("Uniform buffer (set {}, binding {}): {}",
//!                 set.unwrap(), binding.unwrap(), ubo.name.unwrap_or_default(),
//!             );
//!         },
//!         Err(err) => eprintln!("parse error: {err}")
//!     }
//! }
//! for pc in reflector.resources_for_type(ResourceType::PushConstant).unwrap() {
//!     match pc {
//!         Ok(pc) => {
//!             let size = reflector
//!                 .type_description(pc.base_type_id)
//!                 .unwrap().size_hint.declared();
//!             println!("Push constant (size {size}): {}", pc.name.unwrap_or_default());
//!         },
//!         Err(err) => eprintln!("parse error: {err}"),
//!     }
//! }
//! ```

pub mod stream;
pub mod op;
mod module;
pub mod reflect;


mod core;

pub use {
    core::*,
    module::*,
};
