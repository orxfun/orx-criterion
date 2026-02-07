use crate::Treatment;

#[test]
fn treatment_0() {
    pub struct MyTreat;

    impl Treatment<0> for MyTreat {
        fn factor_names() -> [&'static str; 0] {
            []
        }

        fn factor_values(&self) -> [String; 0] {
            []
        }
    }

    let t = MyTreat;

    assert_eq!(t.to_string(), "");
}

#[test]
fn treatment_1() {
    pub struct MyTreat(usize);

    impl Treatment<1> for MyTreat {
        fn factor_names() -> [&'static str; 1] {
            ["width"]
        }

        fn factor_values(&self) -> [String; 1] {
            [self.0.to_string()]
        }
    }

    let t = MyTreat(42);

    assert_eq!(t.to_string(), "width:42");
}

#[test]
fn treatment_3() {
    pub struct MyTreat {
        len: usize,
        sort: bool,
        split: char,
    }

    impl Treatment<3> for MyTreat {
        fn factor_names() -> [&'static str; 3] {
            ["len", "sort", "split"]
        }

        fn factor_values(&self) -> [String; 3] {
            [
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

    assert_eq!(t.to_string(), "len:9876543210_sort:true_split:7");
}
