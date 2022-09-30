use std::pin::Pin;
use std::sync::{Arc, Weak, Mutex};
use std::collections::{HashMap};
use weak_table::WeakHashSet;

use once_cell::sync::Lazy;
// HashMap<reactive_id, HahsMap<reactive_key_path, HashSet<Arc<effect>>>>
pub static BUCKET: Lazy<Mutex<HashMap<usize, HashMap<String, Arc<Mutex<WeakHashSet<Weak<EffectImpl>>>>>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// 当前正在执行的effect
static ACTIVE_EFFECT: Lazy<Mutex<Option<Arc<EffectImpl>>>> = Lazy::new(|| Mutex::new(None));
// 将要执行回调的effects
static STACK_EFFECTS: Lazy<Mutex<WeakHashSet<Weak<EffectImpl>>>> = Lazy::new(|| Mutex::new(WeakHashSet::new()));
static INCREMENT_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

use crate::reactive::ReactiveImpl;

pub struct EffectImpl {
  id: usize,
  deps: Mutex<Vec<Arc<Mutex<WeakHashSet<Weak<EffectImpl>>>>>>,
  closure: Pin<Box<dyn Fn() -> () + Send + Sync + 'static>>
}
impl EffectImpl {
  pub fn track(target: &ReactiveImpl, key: &str) {
    let active_effect = ACTIVE_EFFECT.lock().unwrap();
    if active_effect.is_none() { return }
    let mut bucket = BUCKET.lock().unwrap();
    let reactive_id = target.id;
    if bucket.get(&reactive_id).is_none() {
      bucket.insert(reactive_id, HashMap::new());
    }
    let deps_map = bucket.get_mut(&reactive_id).unwrap();
    if deps_map.get(key).is_none() {
      deps_map.insert(key.into(), Arc::new(Mutex::new(WeakHashSet::new())));
    }
    let dep_set = deps_map.get_mut(key).unwrap();
    let mut dep_set_unlock = dep_set.lock().unwrap();
    let active_effect = active_effect.as_ref().unwrap();
    if !dep_set_unlock.contains(active_effect) {
      dep_set_unlock.insert(active_effect.clone());
      let mut active_effect_deps = active_effect.deps.lock().unwrap();
      active_effect_deps.push(dep_set.clone());
    }
  }
  pub fn trigger(target: &ReactiveImpl, key: &str) {
    let bucket = BUCKET.lock().unwrap();
    let reactive_id = target.id;
    let deps_map = bucket.get(&reactive_id);
    if let Some(deps_map) = deps_map {
      let dep_set = deps_map.get(key);
      if let Some(dep_set) = dep_set {
        let dep_set = dep_set.lock().unwrap();
        for effect in dep_set.iter() {
          let effect = effect.clone();
          {
            let mut stack_effects = STACK_EFFECTS.lock().unwrap();
            if stack_effects.contains(&effect) { // effect是否在将执行队列中
              return
            } else {
              stack_effects.insert(effect.clone());
            }
          }
          std::thread::spawn(move || {
            effect.run();
            let mut stack_effects = STACK_EFFECTS.lock().unwrap();
            stack_effects.remove(&effect);
          });
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
      deps: Mutex::new(vec![]),
      closure: Box::pin(closure)
    });

    {
      let mut active_effect = ACTIVE_EFFECT.lock().unwrap();
      *active_effect = Some(effect.clone());
    }
    ((*effect).closure)();
    {
      let mut active_effect = ACTIVE_EFFECT.lock().unwrap();
      *active_effect = None;
    }
    effect
  }
  pub fn run(&self) {
    (self.closure)();
  }
  fn cleanup(&mut self) {
    let deps = self.deps.lock().unwrap();
    for dep in deps.iter() {
      let mut dep = dep.lock().unwrap();
      dep.remove(self);
    }
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
impl Drop for EffectImpl {
  fn drop(&mut self) {
    log::debug!("drop effect {}", self.id);
    self.cleanup();
  }
}


pub fn effect<F>(closure: F) -> Arc<EffectImpl>
where F: Fn() -> () + Send + Sync + 'static
{
  EffectImpl::new(closure)
}