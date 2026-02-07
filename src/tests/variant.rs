use crate::Variant;

#[test]
fn treatment_0() {
    pub struct MyVariant;

    impl Variant<0> for MyVariant {
        fn param_names() -> [&'static str; 0] {
            []
        }

        fn param_values(&self) -> [String; 0] {
            []
        }
    }

    let t = MyVariant;

    assert_eq!(t.to_string(), "");
}

#[test]
fn treatment_1() {
    pub struct MyVariant(usize);

    impl Variant<1> for MyVariant {
        fn param_names() -> [&'static str; 1] {
            ["width"]
        }

        fn param_values(&self) -> [String; 1] {
            [self.0.to_string()]
        }
    }

    let t = MyVariant(42);

    assert_eq!(t.to_string(), "width:42");
}

#[test]
fn treatment_3() {
    pub struct MyVariant {
        len: usize,
        sort: bool,
        split: char,
    }

    impl Variant<3> for MyVariant {
        fn param_names() -> [&'static str; 3] {
            ["len", "sort", "split"]
        }

        fn param_values(&self) -> [String; 3] {
            [
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
