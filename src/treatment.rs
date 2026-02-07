pub trait Treatment<const N: usize> {
    fn factor_names() -> [&'static str; N];

    fn factor_values(&self) -> [String; N];

    fn to_string(&self) -> String {
        to_string(&Self::factor_names(), &self.factor_values())
    }
}

pub(super) fn to_string(names: &[&'static str], values: &[String]) -> String {
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
