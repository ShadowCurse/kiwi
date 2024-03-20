#![feature(associated_type_defaults)]
#![feature(const_type_name)]
#![feature(const_mut_refs)]
#![feature(const_type_id)]
#![feature(concat_idents)]
#![feature(allocator_api)]
#![feature(fn_traits)]
#![allow(internal_features)]
#![feature(core_intrinsics)]

pub mod archetype;
pub mod blobvec;
pub mod component;
pub mod entity;
pub mod events;
pub mod query;
pub mod resources;
pub mod sparse_set;
pub mod system;
pub mod table;
pub mod utils;
pub mod world;
