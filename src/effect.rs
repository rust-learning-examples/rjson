use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
// static REACTIVES: Lazy<Mutex<HashMap<usize, Arc<RwLock<crate::reactive::ReactiveImpl>>>>> = Lazy::new(||  Mutex::new(HashMap::new()));
static BUCKET: Lazy<Mutex<HashMap<usize, HashMap<String, HashSet<Arc<EffectImpl>>>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static ACTIVE_EFFECT: Lazy<Mutex<Option<Arc<EffectImpl>>>> = Lazy::new(|| Mutex::new(None));
static STACK_EFFECTS: Lazy<Mutex<Vec<Arc<EffectImpl>>>> = Lazy::new(|| Mutex::new(vec![]));
static INCREMENT_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

use crate::reactive::ReactiveImpl;

pub struct EffectImpl {
  id: usize,
  closure: Pin<Box<dyn Fn() -> () + Send + Sync + 'static>>
}
impl EffectImpl {
  pub fn track(target: &ReactiveImpl, key: &str) {
    // let target = target.read().unwrap();
    // let mut reactvies = REACTIVES.lock().unwrap();
    let active_effect = ACTIVE_EFFECT.lock().unwrap();
    if active_effect.is_none() { return }
    let mut bucket = BUCKET.lock().unwrap();
    let reactive_id = target.id;
    // if reactvies.get(&reactive_id).is_none() {
    //   reactvies.insert(reactive_id, target.clone());
    // }
    if bucket.get(&reactive_id).is_none() {
      bucket.insert(reactive_id, HashMap::new());
    }
    let deps_map = bucket.get_mut(&reactive_id).unwrap();
    if deps_map.get(key).is_none() {
      deps_map.insert(key.into(), HashSet::new());
    }
    let dep_set = deps_map.get_mut(key).unwrap();
    let active_effect = active_effect.as_ref().unwrap();
    if !dep_set.contains(active_effect) {
      dep_set.insert(active_effect.clone());
    }
  }
  pub fn trigger(target: Arc<RwLock<ReactiveImpl>>, key: &str) {
    let target = target.read().unwrap();
    // let reactvies = REACTIVES.lock().unwrap();
    let bucket = BUCKET.lock().unwrap();
    let reactive_id = target.id;
    // if reactvies.get(&reactive_id).is_none() {
    //   return
    // }
    let deps_map = bucket.get(&reactive_id);
    if let Some(deps_map) = deps_map {
      let dep_set = deps_map.get(key);
      if let Some(dep_set) = dep_set {
        for effect in dep_set.iter() {
          effect.run();
        }
      }
    }
  }
  pub fn new<F>(closure: F) -> Arc<Self>
  where F: Fn() -> () + Send + Sync + 'static
  {
    let mut increment_counter = INCREMENT_COUNTER.lock().unwrap();
    *increment_counter += 1;
    let effect = Arc::new(Self {
      id: *increment_counter,
      closure: Box::pin(closure)
    });

    {
      let mut stack_effects = STACK_EFFECTS.lock().unwrap();
      let mut active_effect = ACTIVE_EFFECT.lock().unwrap();
      *active_effect = Some(effect.clone());
      stack_effects.push(effect.clone());
    }
    ((*effect).closure)();
    {
      let mut stack_effects = STACK_EFFECTS.lock().unwrap();
      let mut active_effect = ACTIVE_EFFECT.lock().unwrap();
      stack_effects.pop().unwrap();
      *active_effect = None;
    }
    effect
  }
  pub fn run(&self) {
    (self.closure)();
  }
}
impl core::hash::Hash for EffectImpl {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}
impl std::cmp::PartialEq for EffectImpl {
  fn eq(&self, other: &Self) -> bool {
      self.id == other.id
  }
}
impl std::cmp::Eq for EffectImpl {
}


pub fn effect<F>(closure: F) -> Arc<EffectImpl>
where F: Fn() -> () + Send + Sync + 'static
{
  EffectImpl::new(closure)
}