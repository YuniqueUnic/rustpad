use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
};

#[derive(Debug)]
struct CellPoint {
    // CellPoint is not thread safe due to it can used on more situations
    // and it's performance is lower than Point that CellPoint wrapped by the Cell
    // which has some checks & operations during runtime
    x: Cell<i32>,
    y: Cell<i32>,
}

#[derive(Debug)]
struct Point {
    // Point is thread safe because it limited by the ownership rule directly,
    // and the performance of Point is good than Cell wrapped type.
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
    fn move_by(&mut self, dx: i32, dy: i32) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
    fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl CellPoint {
    fn new(x: i32, y: i32) -> Self {
        CellPoint {
            x: Cell::new(x),
            y: Cell::new(y),
        }
    }
    fn move_by(&self, dx: i32, dy: i32) {
        self.x.set(self.x.get() + dx);
        self.y.set(self.y.get() + dy);
    }
    fn get_position(&self) -> (i32, i32) {
        (self.x.get(), self.y.get())
    }
}
fn point_test() {
    let point = CellPoint::new(0, 0);
    point.move_by(5, 10);
    println!("{:?}", point.get_position()); // 输出：(5, 10)
}

pub fn cell_test() {
    let c = Counter::new();
    dbg!(&c); // last_value: -1, value: 0
    c.increment();
    dbg!(&c); //  last_value: -1, value: 1
    let mut c = c;
    c.inc_last_value();
    c.increment();
    dbg!(&c); //  last_value: 0, value: 2
    let mut c = Cell::new(c);
    c.get_mut().inc_last_value();
    c.borrow_mut().get_mut().increment();
    dbg!(c.get_mut()); //  last_value: 1, value: 3
    let c = RefCell::new(c.into_inner());
    c.borrow().increment();
    c.borrow_mut().inc_last_value();
    // let c_ref = c.borrow();
    // c_ref.increment();
    // let mut c_mut_ref = c.borrow_mut();  // already borrowed: BorrowMutError
    // c_mut_ref.inc_last_value();
    dbg!(c.borrow()); //  last_value: 2, value: 4

    let a = RefCell::new(1);
    let a_mut_ref = a.borrow_mut();
    // drop(a_mut_ref);
    let a_ref = a.borrow();
    // println!("{}", a_mut_ref);
}

#[derive(Debug)]
struct Counter {
    lastvalue: i32,
    value: Cell<i32>,
}

impl Counter {
    fn new() -> Counter {
        Counter {
            lastvalue: -1,
            value: Cell::new(0),
        }
    }

    fn increment(&self) {
        // 完成这个函数，使得每次调用都会使 value 增加 1
        self.value.set(self.value.get() + 1);
    }

    fn get_value(&self) -> i32 {
        // 完成这个函数，返回当前的 value
        self.value.get()
    }

    fn inc_last_value(&mut self) {
        self.lastvalue += 1;
    }
}
