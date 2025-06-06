use color_eyre::{
    eyre::ContextCompat,
    Result,
};

static ERROR_MESSAGE_YAML: &str = "failed merging yaml documents";

pub fn yaml(a: &mut serde_yaml::Value, b: serde_yaml::Value) -> Result<()> {
    match (a, b) {
        (a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Mapping(b)) => {
            let a = a.as_mapping_mut().wrap_err(ERROR_MESSAGE_YAML)?;
            for (k, v) in b {
                if v.is_sequence() && a.contains_key(&k) && a[&k].is_sequence() {
                    let mut b2 = a
                        .get(&k)
                        .wrap_err(ERROR_MESSAGE_YAML)?
                        .as_sequence()
                        .wrap_err(ERROR_MESSAGE_YAML)?
                        .to_owned();
                    b2.append(&mut v.as_sequence().wrap_err(ERROR_MESSAGE_YAML)?.to_owned());
                    a[&k] = serde_yaml::Value::from(b2);
                    continue;
                }
                if a.contains_key(&k) {
                    yaml(&mut a[&k], v)?;
                } else {
                    a.insert(k.clone(), v.clone());
                }
            }
        }
        (a, b) => *a = b,
    }
    Ok(())
}
