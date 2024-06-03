/*
第一题
    1. 使用 if 检查 i32 类型的变量是否为正数、负数或零，并打印相应的信息。
    2. 使用 loop 编写一个无限循环，当循环次数达到 10 次时，使用 break 退出循环。
    3. 使用 for 循环遍历 1 到 888 的数字，并只打印出其中的偶数。
第二题
    1. 编写一个函数，对一个 i32 类型的数组进行冒泡排序。
    要求
    接收一个 i32 类型的数组参数，函数返回一个排好序的数组
    接收一个 bool 类型的参数，根据该参数决定是升序还是降序
    2. 编写一个函数，在一个 i32 类型的数组中查找给定的值，并返回结果。
    要求
    先用上一个排序函数，对数组进行排序
    函数返回一个元组，元组第一个分量表示待找元素是否存在，第二个分量表示其在数组中的位置
第三题
    编写一个函数，生成 n 阶的斐波那契数列。
*/

use std::{
    io::{Error, ErrorKind},
    result::Result,
};

// The question 1
pub fn question1(num: i32) {
    match check_i32(num) {
        Some(is_mins) => {
            let s = if is_mins { "positive" } else { "negative" };
            println!("This item:{} is a {} number", &num, s);
        }
        None => println!("This item: {} is zero", &num),
    }

    let mut loop_times: u8 = 10;
    loop {
        if loop_times == 0 {
            break;
        }
        println!("{}", loop_times);
        loop_times -= 1;
    }

    for n in 1..=888 {
        if n / 2 == 0 {
            println!("{}", n);
        }
    }
}

fn check_i32(num: i32) -> Option<bool> {
    if num == 0 {
        return None;
    }

    if num > 0 {
        return Some(true);
    } else {
        return Some(false);
    }
}

// The question 2
pub fn question2(num: i32, nums: &mut [i32], ascend: bool) {
    // 1.
    let nums = Vec::from(nums);
    let mut test_nums = nums.clone();
    let mut base_nums = nums.clone();
    bubble_sort(&mut test_nums, ascend);
    base_nums.sort_by(|a, b| if ascend { a.cmp(b) } else { b.cmp(a) });
    println!("The sorted nums: {:?}", test_nums);
    assert_eq!(
        base_nums, test_nums,
        "we are testing bubble_sort with base:{:?} and actual:{:?}",
        base_nums, test_nums
    );

    // 2.
    match look_num(num, &nums[..]) {
        Ok((found, indices)) => {
            println!(
                "The num:{} is found in the nums:{:?}? The answer is {}",
                num, nums, found
            );
            println!("So, the indices of it in the nums is {:?}", indices);
        }
        Err(e) => {
            eprintln!("Something wrong:{:?}", e);
        }
    };
}

fn bubble_sort(nums: &mut [i32], ascend: bool) {
    let len = nums.len();
    for i in (0..=len).rev() {
        for n in 1..i {
            if nums[n] < nums[n - 1] {
                if ascend {
                    nums.swap(n, n - 1);
                }
            } else {
                if !ascend {
                    nums.swap(n, n - 1);
                }
            }
        }
    }
}

fn look_num(num: i32, nums: &[i32]) -> Result<(bool, Vec<usize>), Error> {
    let mut nums = Vec::from(nums);
    bubble_sort(&mut nums, true);
    let mut indcies: Vec<usize> = Vec::new();
    for (i, &n) in nums.iter().enumerate() {
        if num == n {
            indcies.push(i);
        }
    }
    Ok((indcies.is_empty(), indcies))
}

pub fn question3(level: usize) {
    let result = fibonacci(level);
    println!("The result of {:?} level of fibonacci: {:?}", level, result);
}

fn fibonacci(level: usize) -> Result<Vec<u128>, Error> {
    let mut fibo: Vec<u128> = Vec::new();
    if level < 1 {
        eprintln!("The level num cannot less than 1");
        return Err(ErrorKind::InvalidData.into());
    }

    match level {
        1 => fibo.push(1),
        2 => fibo.extend_from_slice(&[1, 1]),
        3 => fibo.extend_from_slice(&[1, 1, 2]),
        _ => {
            fibo.extend_from_slice(&[1, 1, 2]);
            // level >= 4
            for i in 3..=level - 1 {
                let next = fibo[i - 2] + fibo[i - 1];
                fibo.push(next);
            }
        }
    }

    return Ok(fibo);
}
