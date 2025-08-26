pub fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), Value::Object(b)) => {
            for (k, v) in b {
                // preventing null copy
                if !v.is_null() {
                    merge(a.entry(k.clone()).or_insert(Value::Null), v);
                }
            }
        }
        // override if a and b is not a object
        (a, b) => {
            *a = b.clone();
        }
    }
}
