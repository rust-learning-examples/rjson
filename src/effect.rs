use crate::RJson;
use std::pin::Pin;
use std::sync::{mpsc, Arc, Mutex, Weak};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use weak_table::WeakHashSet;
pub static BUCKET: Lazy<
    Mutex<HashMap<String, HashMap<String, Arc<Mutex<WeakHashSet<Weak<Effect>>>>>>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));
// 当前正在执行的effect
static ACTIVE_EFFECT: Lazy<Mutex<Option<Arc<Effect>>>> = Lazy::new(|| Mutex::new(None));

lazy_static::lazy_static! {
    static ref EFFECT_RUNNER: EffectRunner = {
        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));
        EffectRunner {
            sender: Some(sender),
            thread: Some(std::thread::spawn(move || {
                // 延迟1s执行effect.run方法
                let delay = std::time::Duration::from_millis(1);
                let effect_run_debouncer = debounce::EventDebouncer::new(delay, move |effect: Arc<Effect>| {
                    effect.run();
                });
                loop {
                    let message = receiver.recv();
                    match message {
                        Ok(effect) => {
                            effect_run_debouncer.put(effect);
                        }
                        Err(_) => {
                            println!("shutting down.");
                            break;
                        }
                    }
                }
            }))
        }
    };
}

/**
 * EffectRunner
 */
struct EffectRunner {
    sender: Option<Arc<Mutex<mpsc::Sender<Arc<Effect>>>>>,
    thread: Option<std::thread::JoinHandle<()>>
}
impl EffectRunner {
    fn run(&self, effect: Arc<Effect>) {
        self.sender.as_ref().unwrap().lock().unwrap().send(effect).unwrap();
    }
}
impl Drop for EffectRunner {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
/**
 * Effect
 */
pub struct Effect {
    closure: Pin<Box<dyn Fn() -> () + Send + Sync + 'static>>,
    deps: Mutex<Vec<Arc<Mutex<WeakHashSet<Weak<Effect>>>>>>,
}

impl Effect {
    pub fn track(target: &serde_json::Value, key: &str) {
        let active_effect = ACTIVE_EFFECT.lock().unwrap();
        if active_effect.is_none() {
            return;
        }
        let mut bucket = BUCKET.lock().unwrap();
        let reactive_ptr = target.get_ptr();
        if bucket.get(&reactive_ptr).is_none() {
            bucket.insert(reactive_ptr.clone(), HashMap::new());
        }
        let deps_map = bucket.get_mut(&reactive_ptr).unwrap();
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
    pub fn trigger(target: &serde_json::Value, key: &str) {
        let bucket = BUCKET.lock().unwrap();
        let reactive_ptr = target.get_ptr();
        let deps_map = bucket.get(&reactive_ptr);
        if let Some(deps_map) = deps_map {
            let dep_set = deps_map.get(key);
            if let Some(dep_set) = dep_set {
                let dep_set = dep_set.lock().unwrap();
                for effect in dep_set.iter() {
                    let effect = effect.clone();
                    EFFECT_RUNNER.run(effect);
                }
            }
        }
    }
    pub fn new<F>(closure: F) -> Arc<Self>
    where
        F: Fn() -> () + Send + Sync + 'static,
    {
        let effect = Arc::new(Self {
            closure: Box::pin(closure),
            deps: Mutex::new(vec![]),
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
    fn get_ptr(&self) -> String {
        format!("{:p}", self)
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
impl core::hash::Hash for Effect {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get_ptr().hash(state);
    }
}
impl std::cmp::PartialEq for Effect {
    fn eq(&self, other: &Self) -> bool {
        self.get_ptr() == other.get_ptr()
    }
}
impl std::cmp::Eq for Effect {}
impl Drop for Effect {
    fn drop(&mut self) {
        log::debug!("drop effect {:?}", self.get_ptr());
        self.cleanup();
    }
}