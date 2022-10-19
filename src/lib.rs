mod effect;
mod reactive;
pub use effect::Effect;
pub use reactive::Reactive;

use std::sync::{Arc};


pub trait RJson: Sized {
    // fn get_ptr(&self) -> String { format!("{:p}", self) }
    fn get_ptr(&self) -> usize;
    // fn pget<I: serde_json::value::Index>(&self, index: I) -> &serde_json::Value;
    fn pget(&self, index: &str) -> &Self;
    fn g(&self, index: &str) -> &Self { self.pget(index) }
    fn pget_mut(&mut self, index: &str) -> &mut Self;
    fn g_mut(&mut self, index: &str) -> &mut Self { self.pget_mut(index) }
    // fn pset<I: serde_json::value::Index>(&mut self, index: I, value: serde_json::Value);
    fn pset(&mut self, index: &str, value: serde_json::Value);
    fn s(&mut self, index: &str, value: serde_json::Value) { self.pset(index, value) }
}

pub fn reactive<'a>(json: serde_json::Value) -> Arc<Reactive<'a>> {
    Arc::new(Reactive::new(json))
}

pub fn effect<F>(closure: F) -> Arc<Effect>
where
    F: Fn() -> () + Send + Sync + 'static,
{
    Effect::new(closure)
}
