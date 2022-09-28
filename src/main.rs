use rjson::reactive;
fn main() {
  let state = reactive(serde_json::json!({
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

  {
    let state = state.clone();
    rjson::effect(move || {
      let state_inner = state.inner();
      let state_reader = state_inner.read().unwrap();
      println!("hello effect, age: {}", state_reader.pget("age"));
    });
  }

  {
    state.pset("name", "zhangsan".into());
    state.pset("age", 18.into());
    state.pset("age2", serde_json::json!(null));
    state.pset("phones.1", "0539".into());

    let state_inner = state.inner();
    let state_reader = state_inner.read().unwrap();
    println!("name: {}, age: {}, age2: {}", state_reader.pget("name"), state_reader.pget("age"), state_reader.pget("age2"));
    println!("phones: {:?}", state_reader.pget("phones.0"));
  }
}