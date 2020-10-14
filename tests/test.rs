use enhanced_enum::enhanced_enum;
use std::convert::TryFrom;

#[test]
fn test() {
    enhanced_enum!(FooBar { Foo, Bar });

    let x: String = FooBar::Foo.to_string();
    let y: &str = FooBar::Foo.to_str();
    assert_eq!(x, y);

    println!("{:?}", FooBar::Foo);
    println!("{}", FooBar::Foo);

    assert!(FooBar::try_from(1u32).is_ok());
    assert!(FooBar::try_from(5u64).is_err());

    let mut q = FooBarArray::new(7.0);
    assert!(q.contains(&7.0));
    q[FooBar::Foo] = f64::NAN;
    assert!(q.iter().any(|x| x.is_nan()));

    let mut raboof = FooBarArray::new_with(|x| match x {
        FooBar::Foo => 4734,
        FooBar::Bar => 51,
    });
    dbg!(&raboof);
    assert_eq!(raboof[FooBar::Bar], 51);
    for _x in &raboof {}
    for _x in &mut raboof {}
    assert!(!raboof.is_empty());
    assert!(raboof.len() == 2);
    let boofoo = raboof.clone();
    assert!(boofoo == raboof);
    let _vv = FooBarArray::new(vec![6; 7]);
}
