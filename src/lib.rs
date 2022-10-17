mod effect;
pub use effect::Effect;

use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref NUM_REGEX: regex::Regex = {
        regex::Regex::new(r"^\d+$").unwrap()
    };
}
pub trait RJson {
    // fn get_ptr(&self) -> String { format!("{:p}", self) }
    fn get_ptr(&self) -> usize;
    // fn pget<I: serde_json::value::Index>(&self, index: I) -> &serde_json::Value;
    fn pget(&self, index: &str) -> &serde_json::Value;
    // fn pset<I: serde_json::value::Index>(&mut self, index: I, value: serde_json::Value);
    fn pset(&mut self, index: &str, value: serde_json::Value);
}
impl RJson for serde_json::Value {
    fn get_ptr(&self) -> usize {
        // unsafe { std::mem::transmute(&*self) } 
        self as *const serde_json::Value as usize
    }
    fn pget(&self, index: &str) -> &serde_json::Value {
        let indexs: Vec<&str> = index.split(".").collect();
        let mut json = self;
        for index in indexs.into_iter() {
            // track
            crate::effect::Effect::track(json, index);
            if json.is_array() && NUM_REGEX.is_match(index) {
                let index = index.parse::<usize>().unwrap();
                json = &json[index];
            } else {
                json = &json[index];
            }
        }
        json
    }
    fn pset(&mut self, index: &str, value: serde_json::Value) {
        let num_regex = regex::Regex::new(r"^\d+$").unwrap();
        let indexs: Vec<&str> = index.split(".").collect();
        let mut json = self;
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
            // trigger
            crate::effect::Effect::trigger(json, index);
        }
    }
}

pub fn reactive(json: serde_json::Value) -> Arc<Mutex<serde_json::Value>> {
    Arc::new(Mutex::new(json))
}

pub fn effect<F>(closure: F) -> Arc<Effect>
where
    F: Fn() -> () + Send + Sync + 'static,
{
    Effect::new(closure)
}
