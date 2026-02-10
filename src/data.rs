pub trait Data {
    fn factor_names() -> Vec<&'static str>;

    fn factor_values(&self) -> Vec<String>;

    fn factor_names_short() -> Vec<&'static str> {
        Self::factor_names()
    }

    fn factor_values_short(&self) -> Vec<String> {
        self.factor_values()
    }

    fn key_long(&self) -> String {
        join(&Self::factor_names(), &self.factor_values())
    }

    fn key_short(&self) -> String {
        join(&Self::factor_names_short(), &self.factor_values_short())
    }
}

pub(super) fn join(names: &[&'static str], values: &[String]) -> String {
    debug_assert_eq!(names.len(), values.len());
    match names.len() {
        0 => Default::default(),
        1 => format!("{}:{}", names[0], values[0]),
        n => {
            let mut s = String::new();
            s.push_str(&format!("{}:{}", names[0], values[0]));
            for i in 1..n {
                s.push_str(&format!("_{}:{}", names[i], values[i]));
            }
            s
        }
    }
}
