```toml
rjson = { version: "0.0.1", package = "reactive_json" }
```

```rust
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
        effect_state.with_lock(|state| {
            println!("-- hello effect, phones.1: {}", state.pget("phones.1"));
        });
      });

      state.with_lock(|mut state| {
        state.pset("name", "zhangsan".into());
        state.pset("age", 18.into());
        state.pset("age", 19.into());
        /*
         * 无法追加新的属性，会使内存布局重排，导致其他变量ptr地址改变
         * 目前解决方案：通过JSON_ADDR_MAP记录旧地址，当下次访问地址变化，更新为新地址（以存放地址的变量地址作为targetkey）
         */
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
      });
  }

  std::thread::park();
}

```