use crate::Treatment;

#[test]
fn treatment_0() {
    pub struct MyTreat;

    impl Treatment for MyTreat {
        fn factor_names() -> Vec<&'static str> {
            vec![]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![]
        }
    }

    let t = MyTreat;

    assert_eq!(t.to_str_long(), "");
}

#[test]
fn treatment_1() {
    pub struct MyTreat(usize);

    impl Treatment for MyTreat {
        fn factor_names() -> Vec<&'static str> {
            vec!["width"]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![self.0.to_string()]
        }
    }

    let t = MyTreat(42);

    assert_eq!(t.to_str_long(), "width:42");
}

#[test]
fn treatment_3() {
    pub struct MyTreat {
        len: usize,
        sort: bool,
        split: char,
    }

    impl Treatment for MyTreat {
        fn factor_names() -> Vec<&'static str> {
            vec!["len", "sort", "split"]
        }

        fn factor_values(&self) -> Vec<String> {
            vec![
                self.len.to_string(),
                self.sort.to_string(),
                self.split.to_string(),
            ]
        }
    }

    let t = MyTreat {
        len: 9876543210,
        sort: true,
        split: '7',
    };

    assert_eq!(t.to_str_long(), "len:9876543210_sort:true_split:7");
}
