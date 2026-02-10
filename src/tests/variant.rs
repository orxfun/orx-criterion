use crate::AlgFactors;

#[test]
fn variant_0() {
    pub struct MyVariant;

    impl AlgFactors for MyVariant {
        fn factor_names() -> Vec<&'static str> {
            vec![]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![]
        }
    }

    let t = MyVariant;

    assert_eq!(t.key_long(), "");
}

#[test]
fn variant_1() {
    pub struct MyVariant(usize);

    impl AlgFactors for MyVariant {
        fn factor_names() -> Vec<&'static str> {
            vec!["width"]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![self.0.to_string()]
        }
    }

    let t = MyVariant(42);

    assert_eq!(t.key_long(), "width:42");
}

#[test]
fn variant_3() {
    pub struct MyVariant {
        len: usize,
        sort: bool,
        split: char,
    }

    impl AlgFactors for MyVariant {
        fn factor_names() -> Vec<&'static str> {
            vec!["len", "sort", "split"]
        }

        fn factor_names_short() -> Vec<&'static str> {
            vec!["l", "srt", "sp"]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![
                self.len.to_string(),
                self.sort.to_string(),
                self.split.to_string(),
            ]
        }

        fn factor_values_short(&self) -> Vec<String> {
            vec![
                self.len.to_string(),
                match self.sort {
                    true => "T".to_string(),
                    false => "F".to_string(),
                },
                self.split.to_string(),
            ]
        }
    }

    let t = MyVariant {
        len: 9876543210,
        sort: true,
        split: '7',
    };

    assert_eq!(t.key_long(), "len:9876543210_sort:true_split:7");
    assert_eq!(t.key_short(), "l:9876543210_srt:T_sp:7");
}
