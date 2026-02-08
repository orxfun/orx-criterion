use crate::Variant;

#[test]
fn variant_0() {
    pub struct MyVariant;

    impl Variant for MyVariant {
        fn param_names() -> Vec<&'static str> {
            vec![]
        }

        fn param_values(&self) -> Vec<String> {
            vec![]
        }
    }

    let t = MyVariant;

    assert_eq!(t.to_string(), "");
}

#[test]
fn variant_1() {
    pub struct MyVariant(usize);

    impl Variant for MyVariant {
        fn param_names() -> Vec<&'static str> {
            vec!["width"]
        }

        fn param_values(&self) -> Vec<String> {
            vec![self.0.to_string()]
        }
    }

    let t = MyVariant(42);

    assert_eq!(t.to_string(), "width:42");
}

#[test]
fn variant_3() {
    pub struct MyVariant {
        len: usize,
        sort: bool,
        split: char,
    }

    impl Variant for MyVariant {
        fn param_names() -> Vec<&'static str> {
            vec!["len", "sort", "split"]
        }

        fn param_values(&self) -> Vec<String> {
            vec![
                self.len.to_string(),
                self.sort.to_string(),
                self.split.to_string(),
            ]
        }
    }

    let t = MyVariant {
        len: 9876543210,
        sort: true,
        split: '7',
    };

    assert_eq!(t.to_string(), "len:9876543210_sort:true_split:7");
}
