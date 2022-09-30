#[cfg(test)]
mod reactive {
  use std::sync::{Arc, Mutex};
  #[test]
  fn main_test() {
    let state = rjson::reactive(serde_json::json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ],
        "address": {
            "province": "山东"
        }
    }));
    let effect_run_times = Arc::new(Mutex::new(0));
    let _eff = {
      let effect_run_times = effect_run_times.clone();
      let state = state.clone();
      rjson::effect(move || {
          let state = state.lock().unwrap();
          println!("-- hello effect, age: {}", state.pget("age"));
          *effect_run_times.lock().unwrap() += 1;
      })
    };
    assert_eq!(*effect_run_times.lock().unwrap(), 1);

    {
      let mut state = state.lock().unwrap();
      state.pset("name", "zhangsan".into());
      state.pset("age", 18.into());
      state.pset("age", 19.into());
      state.pset("age2", serde_json::json!(null));
      state.pset("phones.1", "0539".into());
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    assert_eq!(*effect_run_times.lock().unwrap(), 2);
  }
}