
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ReactiveImpl(serde_json::Value);
impl ReactiveImpl {
  pub fn get<I>(&self, index: I) -> &serde_json::Value
  where I: serde_json::value::Index {
    &self.0[index]
  }
  pub fn pget(&self, index: &str) -> &serde_json::Value {
    let num_regex = regex::Regex::new(r"^\d+$").unwrap();
    let indexs: Vec<&str> = index.split(".").collect();
    let mut json = &self.0;
    for index in indexs.into_iter() {
      if json.is_array() && num_regex.is_match(index) {
        let index = index.parse::<usize>().unwrap();
        json = &json[index];
      } else {
        json = &json[index];
      }
    }
    json
    // self.pget_closure(vec![index], |root_json, indexs| {
    //   if let Some(index) = indexs.last() {
    //     &root_json[index]
    //   } else {
    //     &root_json
    //   }
    // })
  }
  pub fn set<I>(&mut self, index: I, value: serde_json::Value)
  where I: serde_json::value::Index {
    self.0[index] = value;
  }
  pub fn pset(&mut self, index: &str, value: serde_json::Value) {
    let num_regex = regex::Regex::new(r"^\d+$").unwrap();
    let indexs: Vec<&str> = index.split(".").collect();
    let mut json = &mut self.0;
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
  }
}
impl<I> core::ops::Index<I> for ReactiveImpl where I: serde_json::value::Index
{
    type Output = serde_json::Value;

    fn index(&self, index: I) -> &Self::Output {
      &mut self.0.index(index)
    }
}
impl<I> core::ops::IndexMut<I> for ReactiveImpl where I: serde_json::value::Index
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

pub fn reactive(json: serde_json::Value) -> Arc<Mutex<ReactiveImpl>> {
  Arc::new(Mutex::new(ReactiveImpl(json)))
}