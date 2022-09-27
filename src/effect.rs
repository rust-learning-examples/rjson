use std::pin::Pin;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
static ACTIVE_EFFECTS: Lazy<Mutex<Vec<Arc<EffectImpl>>>> = Lazy::new(|| Mutex::new(vec![]));

pub struct EffectImpl {
  closure: Pin<Box<dyn Fn() -> () + Send + Sync + 'static>>
}
impl EffectImpl {
  pub fn new<F>(closure: F) -> Arc<Self>
  where F: Fn() -> () + Send + Sync + 'static
  {
    let effect = Arc::new(Self {
      closure: Box::pin(closure)
    });
    let mut active_effects = ACTIVE_EFFECTS.lock().unwrap();
    active_effects.push(effect.clone());
    ((*effect).closure)();
    active_effects.pop().unwrap();
    effect
  }
}

pub fn effect<F>(closure: F) -> Arc<EffectImpl>
where F: Fn() -> () + Send + Sync + 'static
{
  EffectImpl::new(closure)
}