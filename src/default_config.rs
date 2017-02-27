use super::config::{Config,Value};
use std::collections::HashMap;

pub struct DefaultConfig {
    internal_config: Config
}

impl DefaultConfig {
    pub fn new(conf: Config) -> DefaultConfig {
        DefaultConfig {
            internal_config: conf
        }
    }

    pub fn get_str_or_default(&self, key: &str, default: &str) -> String {
        match self.internal_config.get_str(key) {
            Some(value) => value,
            None => default.to_owned()
        }
    }

    pub fn get_int_or_default(&self, key: &str, default: i64) -> i64 {
        match self.internal_config.get_int(key) {
            Some(value) => value,
            None => default
        }
    }

    pub fn get_float_or_default(&self, key: &str, default: f64) -> f64 {
        match self.internal_config.get_float(key) {
            Some(value) => value,
            None => default
        }
    }

    pub fn get_bool_or_default(&self, key: &str, default: bool) -> bool {
        match self.internal_config.get_bool(key) {
            Some(value) => value,
            None => default
        }
    }

    pub fn get(&self, key_path: &str) -> Option<Value> {
        self.internal_config.get(key_path)
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.internal_config.get_str(key)
    }

    pub fn get_int(&self, key: &str)  -> Option<i64> {
        self.internal_config.get_int(key)
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.internal_config.get_float(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.internal_config.get_bool(key)
    }

    pub fn get_table(&self, key: &str) -> Option<HashMap<String, Value>> {
        self.internal_config.get_table(key)
    }

    pub fn get_array(self, key: &str) -> Option<Vec<Value>> {
        self.internal_config.get_array(key)
    }
}

#[cfg(test)]
mod tests {
    use DefaultConfig;
    use config;

    fn setup_default_config() -> DefaultConfig {
        let mut config = config::Config::new();
        config.set("stringtest", "result").unwrap();
        config.set("i64test", 12).unwrap();
        config.set("f64test", 14.0).unwrap();
        config.set("booltest", false).unwrap();
        let arr: Vec<i64> = vec![3, 2, 1];
        config.set("arraytest", arr).unwrap();
        DefaultConfig::new(config)
    }

    #[test]
    fn test_get_str_or_default() {
        let config = setup_default_config();
        assert_eq!(config.get_str("stringtest").unwrap(), "result");
        assert_eq!(config.get_str_or_default("stringtest", "asd"), "result");
        assert_eq!(config.get_str_or_default("nonexistant", "default"), "default");
    }

    #[test]
    fn test_get_int_or_default() {
        let config = setup_default_config();
        assert_eq!(config.get_int("i64test").unwrap(), 12);
        assert_eq!(config.get_int_or_default("i64test", 10), 12);
        assert_eq!(config.get_int_or_default("nonexistant", 10), 10);
    }

    #[test]
    fn test_get_float_or_default() {
        let config = setup_default_config();
        assert_eq!(config.get_float("f64test").unwrap(), 14.0);
        assert_eq!(config.get_float_or_default("f64test", 10.0), 14.0);
        assert_eq!(config.get_float_or_default("nonexistant", 10.0), 10.0);
    }

    #[test]
    fn test_get_bool_or_default() {
        let config = setup_default_config();
        assert_eq!(config.get_bool("booltest").unwrap(), false);
        assert_eq!(config.get_bool_or_default("booltest", true), false);
        assert_eq!(config.get_bool_or_default("nonexistant", true), true);
    }

    #[test]
    fn test_misc_forwarded() {
        let config = setup_default_config();
        let comp = vec![3,2,1];
        let values: Vec<i64> = config.get_array("arraytest").unwrap().iter().map(|val| val.clone().into_int().unwrap()).collect();
        assert_eq!(values, comp);
    }
}