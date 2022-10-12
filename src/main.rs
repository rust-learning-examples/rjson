use rjson::RJson;
fn main() {
  tracing_subscriber::fmt::init();
  {
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

      let effect_state = state.clone();
      let _eff = rjson::effect(move || {
          let state = effect_state.lock().unwrap();
          println!("-- hello effect, age: {}", state.pget("age"));
      });

      {
        let mut state = state.lock().unwrap();
        state.pset("name", "zhangsan".into());
        state.pset("age", 18.into());
        state.pset("age", 19.into());
        state.pset("age2", serde_json::json!(null));
        state.pset("phones.1", "0539".into());

        println!(
            "name: {}, age: {}, age2: {}",
            state.pget("name"),
            state.pget("age"),
            state.pget("age2")
        );
        println!("phones: {:?}", state.pget("phones.0"));
        println!("first phone {}", state["phones"][0]);
      }
  }

  std::thread::park();
}
