use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
};

enum List<T> {
    Cons(T, RefCell<Box<List<T>>>),
    Nil,
}

use List::*;

fn main() {
    let list = Cons(
        1,
        RefCell::new(Box::new(Cons(2, RefCell::new(Box::new(Nil))))),
    );

    // 完成以下函数，将一个新的元素追加到链表的末尾
    fn append(list: &RefCell<List<i32>>, elem: i32) {
        // 你的代码
    }

    // 测试追加元素
    // append(&list, 3);
}
