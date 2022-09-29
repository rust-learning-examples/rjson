
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
static INCREMENT_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[derive(Debug, Clone)]
pub struct ReactiveImpl {
  pub id: usize,
  json: serde_json::Value,
}
impl ReactiveImpl {
  pub fn new(json: serde_json::Value) -> Self {
    let mut increment_counter = INCREMENT_COUNTER.lock().unwrap();
    *increment_counter += 1;
    Self {
      id: *increment_counter,
      json
    }
  }
  // pub fn get<I>(&self, index: I) -> &serde_json::Value
  // where I: serde_json::value::Index {
  //   &self.json[index]
  // }
  pub fn pget(&self, index: &str) -> &serde_json::Value {
    // track
    crate::effect::EffectImpl::track(self, index);
    let num_regex = regex::Regex::new(r"^\d+$").unwrap();
    let indexs: Vec<&str> = index.split(".").collect();
    let mut json = &self.json;
    for index in indexs.into_iter() {
      if json.is_array() && num_regex.is_match(index) {
        let index = index.parse::<usize>().unwrap();
        json = &json[index];
      } else {
        json = &json[index];
      }
    }
    json
  }
  // pub fn set<I>(&mut self, index: I, value: serde_json::Value)
  // where I: serde_json::value::Index {
  //   self.json[index] = value;
  // }
  pub fn pset(&mut self, index: &str, value: serde_json::Value) {
    let num_regex = regex::Regex::new(r"^\d+$").unwrap();
    let indexs: Vec<&str> = index.split(".").collect();
    let mut json = &mut self.json;
    for (i, index) in indexs.iter().enumerate() {
      if indexs.len() - 1 > i {
        if json.is_array() && num_regex.is_match(index) {
          let index = index.parse::<usize>().unwrap();
          json = &mut json[index];
        } else {
          json = &mut json[index];
        }
      }
    }
    if let Some(index) = indexs.last() {
      if json.is_array() && num_regex.is_match(index) {
        let index = index.parse::<usize>().unwrap();
        json[index] = value;
      } else {
        json[index] = value;
      }
    }
    // trigger
    crate::effect::EffectImpl::trigger(&self, index);
  }
}
// impl<I> core::ops::Index<I> for ReactiveImpl where I: serde_json::value::Index
// {
//     type Output = serde_json::Value;

//     fn index(&self, index: I) -> &Self::Output {
//       &mut self.json.index(index)
//     }
// }
// impl<I> core::ops::IndexMut<I> for ReactiveImpl where I: serde_json::value::Index
// {
//     fn index_mut(&mut self, index: I) -> &mut Self::Output {
//         self.json.index_mut(index)
//     }
// }
impl core::hash::Hash for ReactiveImpl {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}
impl std::cmp::PartialEq for ReactiveImpl {
  fn eq(&self, other: &Self) -> bool {
      self.id == other.id
  }
}
impl std::cmp::Eq for ReactiveImpl {
}
impl std::ops::Deref for ReactiveImpl {
  type Target = serde_json::Value;
  fn deref(&self) -> &Self::Target {
      &self.json
  }
}
impl std::ops::DerefMut for ReactiveImpl {
  fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.json
  }
}

pub fn reactive(json: serde_json::Value) -> Arc<Mutex<ReactiveImpl>> {
  Arc::new(Mutex::new(ReactiveImpl::new(json)))
}