pub mod generator;
pub mod mapper;
pub mod syllables;

pub use generator::generate_syllables;
pub use mapper::syllable_to_telex;
pub use syllables::Syllable;
