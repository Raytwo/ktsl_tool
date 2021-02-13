/// You thought it'd be a module file, but it was I, Raytwo
/// Jokes aside, all the section structs from ktsl.rs should be moved in their own file here, probably with a Trait to implement at least New.

mod music;
pub use music::*;
mod info;
pub use info::*;
mod padding;
pub use padding::*;
mod sound;
pub use sound::*;
mod unknown;
pub use unknown::*;