use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    fmt::{format, Display},
    rc::Rc,
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

// thread shared counter
#[derive(Debug)]
struct RefCellCounter {
    num: Rc<RefCell<u32>>,
}

impl RefCellCounter {
    pub fn new() -> Self {
        Self {
            num: Rc::new(RefCell::new(0)),
        }
    }

    fn increment(&self) {
        // dbg!(self.num.as_ptr());
        // let self_num = self.num.clone().borrow_mut();
        // *self_num.borrow_mut() += 1;
        // Error: binary assignment operation `+=` cannot be applied to type `Rc<RefCell<u32>>`rustcClick for full compiler diagnostic
        // refcell.rs(61, 9): cannot use `+=` on type `Rc<RefCell<u32>>`
        //  rc.rs(313, 1): the foreign item type `Rc<RefCell<u32>>` doesn't implement `AddAssign<{integer}>`

        self.num.replace_with(|n| *n + 1);
        // dbg!(self.num.as_ptr());
        // ptr is same
    }

    fn clone_self(&self) -> Self {
        self.increment();
        Self {
            num: self.num.clone(),
        }
    }
}

pub fn ref_cell_counter_test() {
    let rfc = RefCellCounter::new();
    dbg!(&rfc);
    let new_rfc = rfc.clone_self();
    dbg!(&rfc);
    dbg!(&new_rfc);
    let rfc_ptr = &rfc as *const _;
    let new_rfc_ptr = &new_rfc as *const _;
    dbg!(rfc_ptr);
    dbg!(new_rfc_ptr);
    //[examples/cells/refcell.rs:80:5] rfc_ptr = 0x00007ffc3b5ff048
    //[examples/cells/refcell.rs:81:5] new_rfc_ptr = 0x00007ffc3b5ff148
    // the rfc and new_rfc is different entity.
    println!("rfc ptr ref num: {}", Rc::strong_count(&rfc.num));
    println!("new rfc ptr ref num: {}", Rc::strong_count(&new_rfc.num)); // so the num are share in the rfc and new_rfc
                                                                         // rfc ptr ref num: 2
                                                                         // new rfc ptr ref num: 2
    let new_rfc_2 = new_rfc.clone_self();
    dbg!(&rfc);
    dbg!(&new_rfc);
    dbg!(&new_rfc_2);
    println!("rfc ptr ref num: {}", Rc::strong_count(&rfc.num));
    println!("new rfc ptr ref num: {}", Rc::strong_count(&new_rfc.num));
    println!(
        "new rfc ptr 2 ref num: {}",
        Rc::strong_count(&new_rfc_2.num)
    ); // so the num are share in the rfc, new_rfc and new_rfc_2
       // rfc ptr ref num: 3
       // new rfc ptr ref num: 3
       // new rfc 2 ptr ref num: 3
}
