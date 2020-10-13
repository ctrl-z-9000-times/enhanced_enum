use enhanced_enum::enhanced_enum;

#[test]
fn test() {
    enhanced_enum!(FooBar { Foo, Bar });

    dbg!(FooBar::Foo);

    let raboof = FooBarArray::new(|x| match x {
        FooBar::Foo => 4734,
        FooBar::Bar => 51,
    });
    assert_eq!(raboof[FooBar::Bar], 51)
}
