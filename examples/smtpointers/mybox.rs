use std::ops::Deref;

pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn mybox_test() {
    // int_mybox_test();
    str_mybox_test();
}

fn int_mybox_test() {
    let x = 5;
    let mybox = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *mybox, "*mybox:{} should be 5", *mybox);
}

fn str_mybox_test() {
    let m = MyBox::new(String::from("Rust"));
    let hello_closure = |s: &str| println!("Hello, {s}");
    let c = &**m;
    // *m: String -> m: MyBox<String> --> *m = *(m.deref()) = *(&String) = String --> **m: str --> &**m --> &str
    hello_closure(c);
}
