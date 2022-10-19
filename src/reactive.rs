// use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex};
use std::collections::{HashMap};
use once_cell::sync::Lazy;
use std::borrow::{Cow};
use crate::RJson;

// lazy_static::lazy_static! {
//   static ref REACTIVE_COUNTER:AtomicUsize = AtomicUsize::new(1);
// }

lazy_static::lazy_static! {
    static ref NUM_REGEX: regex::Regex = {
        regex::Regex::new(r"^\d+$").unwrap()
    };
}

// ptr: HashMap<index, Box<index_ptr>>Box<ptr>
static JSON_ADDR_MAP: Lazy<Mutex<HashMap<usize, HashMap<String, Box<usize>>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
fn update_json_addr(json: &serde_json::Value, index: &str) -> usize {
    let json_ptr = json.get_ptr();
    let index_json_ptr = json[index].get_ptr();
    let mut before_index_json_ptr = index_json_ptr;
    let index_json_v_addr: usize;
    let mut json_addr_map = JSON_ADDR_MAP.lock().unwrap();
    let json_map = json_addr_map.entry(json_ptr).or_insert(HashMap::new());
    if let Some(index_v) = json_map.get_mut(index) {
        before_index_json_ptr = **index_v;
        if before_index_json_ptr != index_json_ptr {
            **index_v = index_json_ptr;
        }
        index_json_v_addr = &**index_v as * const usize as usize;
    } else {
        let ptr_v = Box::new(index_json_ptr);
        index_json_v_addr = &*ptr_v as * const usize as usize;
        json_map.insert(index.into(), ptr_v);
    }
    // index_json地址发生变化，将index_json为json_map对应的地址进行更新
    if before_index_json_ptr != index_json_ptr {
        if let Some(json_map) = json_addr_map.remove(&before_index_json_ptr) {
          json_addr_map.insert(index_json_ptr, json_map);
        }
    }
    index_json_v_addr
}

fn drop_json_addr(json: &serde_json::Value) {
  let json_ptr = json.get_ptr();
  let json_map;
  {
    let mut json_addr_map = JSON_ADDR_MAP.lock().unwrap();
    json_map = json_addr_map.remove(&json_ptr);
  }
  if let Some(json_map) = json_map {
    for (key, _) in json_map.into_iter() {
      drop_json_addr(json.pget(&key))
    }
  }
}

impl RJson for serde_json::Value {
  fn get_ptr(&self) -> usize {
      // unsafe { std::mem::transmute(&*self) }
      self as *const serde_json::Value as usize
  }
  fn pget(&self, index: &str) -> &Self {
      let indexs: Vec<&str> = index.split(".").collect();
      let mut json = self;
      for index in indexs.into_iter() {
          // track
          let ptr_v_addr = update_json_addr(json, index);
          crate::effect::Effect::track(ptr_v_addr, index);
          // println!("= track n: {}, {}, {}", json, ptr_v_addr, index);
          if json.is_array() && NUM_REGEX.is_match(index) {
              let index = index.parse::<usize>().unwrap();
              json = &json[index];
          } else {
              json = &json[index];
          }
      }
      json
  }
  fn pget_mut(&mut self, index: &str) -> &mut Self {
    let indexs: Vec<&str> = index.split(".").collect();
    let mut json = self;
    for index in indexs.into_iter() {
        // track
        let ptr_v_addr = update_json_addr(json, index);
        crate::effect::Effect::track(ptr_v_addr, index);
        // println!("= track n: {}, {}, {}", json, ptr_v_addr, index);
        if json.is_array() && NUM_REGEX.is_match(index) {
            let index = index.parse::<usize>().unwrap();
            json = &mut json[index];
        } else {
            json = &mut json[index];
        }
    }
    json
    // unsafe { &mut *(json as *mut serde_json::Value as *mut T) as &mut T }
  }
  fn pset(&mut self, index: &str, value: serde_json::Value) {
      let num_regex = regex::Regex::new(r"^\d+$").unwrap();
      let indexs: Vec<&str> = index.split(".").collect();
      let mut json = self;
      for (i, index) in indexs.iter().enumerate() {
          if indexs.len() - 1 > i {
            json = json.pget_mut(index);
          }
      }
      if let Some(index) = indexs.last() {
          let _ptr_v_addr = update_json_addr(json, index);
          if json.is_array() && num_regex.is_match(index) {
              let index = index.parse::<usize>().unwrap();
              json[index] = value;
          } else {
              json[index] = value;
          }
          // trigger
          let ptr_v_addr = update_json_addr(json, index);
          // println!("= trigger: {}, {}, {}", json, ptr_v_addr, index);
          crate::effect::Effect::trigger(ptr_v_addr, index);
      }
  }
}

// impl<'a, T> core::ops::Index<Cow<'a, str>> for T where T : RJson + ToOwned
// {
//     type Output = T;

//     fn index(&self, index: Cow<'a, str>) -> &Self::Output {
//       println!("======================1111111");
//       self.pget(&index)
//     }
// }
// impl<'a, T> core::ops::IndexMut<Cow<'a, str>> for T where T: RJson
// {
//     fn index_mut(&mut self, index: Cow<'a, str>) -> &mut Self::Output {
//       println!("======================22222222");
//       self.pget_mut(&index)
//     }
// }



pub struct Reactive<'a>(Mutex<Cow<'a, serde_json::Value>>);

impl<'a> Reactive<'a> {
  pub fn new(json: serde_json::Value) -> Self {
    Self(Mutex::new(Cow::Owned(json)))
  }
  pub fn lock<F: FnOnce(&serde_json::Value)>(&self, cb: F) {
    let json = self.0.lock().unwrap();
    cb(&*json);
  }
  pub fn lock_mut<F: FnOnce(&mut serde_json::Value)>(&self, cb: F) {
    let mut json = self.0.lock().unwrap();
    cb(json.to_mut());
  }
}

impl<'a> Drop for Reactive<'a> {
  fn drop(&mut self) {
    // {
    //   let json_addr_map = JSON_ADDR_MAP.lock().unwrap();
    //   println!("=========== {:#?}", json_addr_map);
    // }
    self.lock(|state| {
      drop_json_addr(&*state);
    });
    log::debug!("drop reactive {:?}", format!("{:p}", self));
    // {
    //   let json_addr_map = JSON_ADDR_MAP.lock().unwrap();
    //   println!("=========== {:#?}", json_addr_map);
    // }
  }
}




// use std::sync::{Arc, Mutex};

// use once_cell::sync::Lazy;
// static INCREMENT_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

// #[derive(Debug, Clone)]
// pub struct ReactiveImpl {
//   pub id: usize,
//   json: serde_json::Value,
// }
// impl ReactiveImpl {
//   pub fn new(json: serde_json::Value) -> Self {
//     let mut increment_counter = INCREMENT_COUNTER.lock().unwrap();
//     *increment_counter += 1;
//     Self {
//       id: *increment_counter,
//       json
//     }
//   }
//   // pub fn get<I>(&self, index: I) -> &serde_json::Value
//   // where I: serde_json::value::Index {
//   //   &self.json[index]
//   // }
//   pub fn pget(&self, index: &str) -> &serde_json::Value {
//     // track
//     crate::effect::EffectImpl::track(self, index);
//     let num_regex = regex::Regex::new(r"^\d+$").unwrap();
//     let indexs: Vec<&str> = index.split(".").collect();
//     let mut json = &self.json;
//     for index in indexs.into_iter() {
//       if json.is_array() && num_regex.is_match(index) {
//         let index = index.parse::<usize>().unwrap();
//         json = &json[index];
//       } else {
//         json = &json[index];
//       }
//     }
//     json
//   }
//   // pub fn set<I>(&mut self, index: I, value: serde_json::Value)
//   // where I: serde_json::value::Index {
//   //   self.json[index] = value;
//   // }
//   pub fn pset(&mut self, index: &str, value: serde_json::Value) {
//     let num_regex = regex::Regex::new(r"^\d+$").unwrap();
//     let indexs: Vec<&str> = index.split(".").collect();
//     let mut json = &mut self.json;
//     for (i, index) in indexs.iter().enumerate() {
//       if indexs.len() - 1 > i {
//         if json.is_array() && num_regex.is_match(index) {
//           let index = index.parse::<usize>().unwrap();
//           json = &mut json[index];
//         } else {
//           json = &mut json[index];
//         }
//       }
//     }
//     if let Some(index) = indexs.last() {
//       if json.is_array() && num_regex.is_match(index) {
//         let index = index.parse::<usize>().unwrap();
//         json[index] = value;
//       } else {
//         json[index] = value;
//       }
//     }
//     // trigger
//     crate::effect::EffectImpl::trigger(&self, index);
//   }
// }
// // impl<I> core::ops::Index<I> for ReactiveImpl where I: serde_json::value::Index
// // {
// //     type Output = serde_json::Value;

// //     fn index(&self, index: I) -> &Self::Output {
// //       &mut self.json.index(index)
// //     }
// // }
// // impl<I> core::ops::IndexMut<I> for ReactiveImpl where I: serde_json::value::Index
// // {
// //     fn index_mut(&mut self, index: I) -> &mut Self::Output {
// //         self.json.index_mut(index)
// //     }
// // }
// impl core::hash::Hash for ReactiveImpl {
//   fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
//     self.id.hash(state);
//   }
// }
// impl std::cmp::PartialEq for ReactiveImpl {
//   fn eq(&self, other: &Self) -> bool {
//       self.id == other.id
//   }
// }
// impl std::cmp::Eq for ReactiveImpl {
// }
// impl std::ops::Deref for ReactiveImpl {
//   type Target = serde_json::Value;
//   fn deref(&self) -> &Self::Target {
//       &self.json
//   }
// }
// impl std::ops::DerefMut for ReactiveImpl {
//   fn deref_mut(&mut self) -> &mut Self::Target {
//       &mut self.json
//   }
// }
// impl Drop for ReactiveImpl {
//   fn drop(&mut self) {
//     let mut bucket = crate::effect::BUCKET.lock().unwrap();
//     bucket.remove(&self.id);
//     log::debug!("drop reactive {}", self.id);
//   }
// }

// pub fn reactive(json: serde_json::Value) -> Arc<Mutex<ReactiveImpl>> {
//   Arc::new(Mutex::new(ReactiveImpl::new(json)))
// }
