pub trait Treatment<const N: usize> {
    fn factor_names() -> [&'static str; N];

    fn factor_values(&self) -> [String; N];

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match N {
            0 => Ok(()),
            1 => write!(f, "{}:{}", Self::factor_names()[0], self.factor_values()[0]),
            _ => {
                write!(f, "{}:{}", Self::factor_names()[0], self.factor_values()[0])?;
                for _ in 1..N {
                    write!(
                        f,
                        "_{}:{}",
                        Self::factor_names()[0],
                        self.factor_values()[0]
                    )?;
                }
                Ok(())
            }
        }
    }

    fn to_string(&self) -> String {
        match N {
            0 => Default::default(),
            1 => format!("{}:{}", Self::factor_names()[0], self.factor_values()[0]),
            _ => {
                let mut s = String::new();
                s.push_str(&format!(
                    "{}:{}",
                    Self::factor_names()[0],
                    self.factor_values()[0]
                ));
                for i in 1..N {
                    s.push_str(&format!(
                        "_{}:{}",
                        Self::factor_names()[i],
                        self.factor_values()[i]
                    ));
                }
                s
            }
        }
    }
}
