use autometrics::GetLabel;
use autometrics_macros::AutometricsLabel;

#[test]
fn custom_trait_implementation() {
    struct CustomResult;

    impl GetLabel for CustomResult {
        fn get_label(&self) -> Option<(&'static str, &'static str)> {
            Some(("ok", "my-result"))
        }
    }

    assert_eq!(Some(("ok", "my-result")), CustomResult {}.get_label());
}

#[test]
fn manual_enum() {
    enum MyFoo {
        A,
        B,
    }

    impl GetLabel for MyFoo {
        fn get_label(&self) -> Option<(&'static str, &'static str)> {
            Some(("hello", match self {
                MyFoo::A => "a",
                MyFoo::B => "b",
            }))
        }
    }

    assert_eq!(Some(("hello", "a")), MyFoo::A.get_label());
    assert_eq!(Some(("hello", "b")), MyFoo::B.get_label());
}

#[test]
fn derived_enum() {
    #[derive(AutometricsLabel)]
    #[autometrics_label(key = "my_foo")]
    enum MyFoo {
        #[autometrics_label(value = "hello")]
        Alpha,
        #[autometrics_label()]
        BetaValue,
        Charlie,
    }

    assert_eq!(Some(("my_foo", "hello")), MyFoo::Alpha.get_label());
    assert_eq!(Some(("my_foo", "beta_value")), MyFoo::BetaValue.get_label());
    assert_eq!(Some(("my_foo", "charlie")), MyFoo::Charlie.get_label());
}
