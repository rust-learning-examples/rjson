pub(crate) mod reactive;
pub(crate) mod effect;
pub use reactive::reactive;
pub use effect::effect;

pub trait RJson {
    fn pget<I: serde_json::value::Index>(&self, index: I) -> &serde_json::Value;
    fn pset<I: serde_json::value::Index>(&mut self, index: I, value: serde_json::Value);
}

// impl RJson for serde_json::Value {
//     fn pget<I: serde_json::value::Index>(&self, index: I) -> &serde_json::Value {
//         &self[index]
//     }
//     fn pset<I: serde_json::value::Index>(&mut self, index: I, value: serde_json::Value) {
//         self[index] = value
//     }
// }