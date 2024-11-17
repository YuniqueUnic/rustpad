use std::{
    borrow::Borrow,
    cell::RefCell,
    fmt::{format, Display},
};

#[derive(Debug)]
struct Person {
    name: String,
    age: RefCell<u32>,
}

impl Person {
    fn new<S: AsRef<str>>(name: S, age: u32) -> Person {
        Self {
            name: name.as_ref().to_owned(),
            age: RefCell::new(age),
        }
    }

    fn birthday(&self) {
        *self.age.borrow_mut() += 1;
        // let mut p = Person::new(); // the p is mutable
        // *p.age.get_mut() += 1;     // use get_mut to avoid the borrow checker due to he p is mutable obviously.

        // let p = Person::new();
        // *p.age.borrow_mut() += 1;
        // the p is immutable now so that only use borrow_mut can do the inner modification
        // and it only works when the borrow checker said yes during runtime.
    }

    fn get_info(&self) -> String {
        format!("{} is {} years old", self.name, self.age.borrow())
    }
}

pub fn ref_cell_test() {
    let p = Person::new("john", 32);
    dbg!(p.get_info());
    p.birthday();
    dbg!(p.get_info());
}
