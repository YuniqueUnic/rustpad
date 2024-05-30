use rustpad::rustl1;
use std::any;

fn main() {
    // fn_wrapper(hackquest_day2);
}

// this wrapper function can be turned into a declarative macro
// It's hard to rewrite it as a declarative macro
fn fn_wrapper<T>(function: T)
where
    T: FnOnce() + 'static,
{
    // get the actual name of the function
    let name = any::type_name::<T>();
    println!("Start running function: {}", name);
    function();
    println!("Success!");
}

// #[rustpad::log_fn]
fn hackquest_day2() {
    use std::collections::HashMap;
    let v1 = vec![1, 2, 3];
    let v2 = Vec::from([1, 2, 3]);
    assert_eq!(v1, v2);

    let mut v3 = Vec::from([1, 2, 3]);
    for i in &mut v3 {
        *i += 1
    }
    assert_eq!(v3, vec![2, 3, 4]);
    let student_arr: [(&str, i32); 3] = [("Alice", 100), ("Bob", 10), ("Eve", 50)];

    let mut student_map1 = HashMap::new();

    for stu in &student_arr {
        student_map1.insert(stu.0, stu.1);
    }
    let student_map2 = student_arr.into_iter().collect();
    assert_eq!(student_map1, student_map2);
}
