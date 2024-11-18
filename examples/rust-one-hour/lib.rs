const GLOBAL_CONST_VALUE: &'static str = "我是全局常量";
static GLOBAL_STATIC_VALUE: &'static str = "我是全局不可变静态变量";
static mut GLOBAL_STATIC_MUT_VALUE: &'static str = "我是全局可变静态变量";

fn main() {
    owership::struct_methods::struct_methods_test();
}

mod owership {
    pub mod move_ref {
        // Move
        fn consume_entity(e: String) {
            println!("{}", e);
        }

        // ref
        fn ref_entity<T: AsRef<str>>(e: T) {
            println!("{}", e.as_ref());
        }

        // mut ref
        fn ref_entity_mut(e: &mut String) {
            println!("{}", e.as_mut());
            e.push_str("_Mutated");
            println!("{}", e.as_mut());
        }

        pub fn test_move_ref() {
            let s = String::from("hello");
            consume_entity(s);
            //  ref_entity(s); // use of moved value: `s` value used here after move

            let s = String::from("hello");
            let s_1 = s.clone();
            consume_entity(s);
            ref_entity(&s_1);

            let mut s_mut = s_1;
            ref_entity_mut(&mut s_mut);
        }
    }

    pub mod clone_copy {
        // Clone and Copy
        pub fn clone_copy_relationship_0() {
            // Clone
            let clone_type: String = String::from("I'm a clone type which haven't impl the copy");
            let new_clone = clone_type;

            //  println!("{}",clone_type); // use of moved value: `clone_type`, value used here after move
            println!("{}", new_clone);

            // Copy
            let copy_type: usize = 5;
            let new_copy = copy_type;

            println!("{}", copy_type);
            println!("{}", new_copy);
        }

        #[derive(Clone, Debug)]
        struct OnlyClone;

        pub fn clone_copy_relationship_1() {
            let only_clone = OnlyClone;
            // let new_only_clone = only_clone;
            println!("{:?}", only_clone);
            // println!("{:?}", new_only_clone);
        }

        #[derive(Copy, Clone, Debug)]
        struct CloneAndCopy;

        pub fn clone_copy_relationship_2() {
            let clone_and_copy = CloneAndCopy;
            let new_clone_and_copy = clone_and_copy;
            println!("{:?}", clone_and_copy);
            println!("{:?}", new_clone_and_copy);
        }
    }

    pub mod struct_methods {
        use std::{
            fmt::{self, Debug, Display, Formatter},
            ops,
        };

        enum Pot {
            X(i32),
            Y(i32),
        }

        // struct Pot(i32);

        // {:?}
        impl Debug for Pot {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                // write!(f, "Pot X: {}", self.0)
                match self {
                    Pot::X(x) => write!(f, "Pot X: {}", x),
                    Pot::Y(y) => write!(f, "Pot Y: {}", y),
                }
            }
        }

        // Pot + Pot
        impl ops::AddAssign<Pot> for Pot {
            fn add_assign(&mut self, rhs: Pot) {
                // self.0 += rhs.0;
                match self {
                    Pot::X(x) => match rhs {
                        Pot::X(y) => *x += y,
                        Pot::Y(_) => panic!("Cannot add Y to X"),
                    },
                    Pot::Y(y) => match rhs {
                        Pot::X(_) => unreachable!("Cannot add X to Y"),
                        Pot::Y(z) => *y += z,
                    },
                }
            }
        }

        // Pot + i32
        impl ops::AddAssign<i32> for Pot {
            fn add_assign(&mut self, rhs: i32) {
                // self.0 += rhs;
                match self {
                    Pot::X(x) => *x += rhs,
                    Pot::Y(y) => *y += rhs,
                }
            }
        }

        #[derive(Debug)]
        struct Point {
            x: Pot,
            y: Pot,
        }

        impl Point {
            // Associated function: 关联函数
            fn new(x: i32, y: i32) -> Self {
                Point {
                    x: Pot::X(x),
                    y: Pot::Y(y),
                }
            }

            // Instance method: 实例方法
            fn move_by(&mut self, dx: i32, dy: i32) {
                self.x += dx; // impl ops::AddAssign<i32> for Pot
                self.y += dy;
                println!("{:?}", self);
            }
        }

        impl Display for Pot {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Pot::X(x) => write!(f, "Pot X: {}", x),
                    Pot::Y(y) => write!(f, "Pot Y: {}", y),
                }
            }
        }

        // 实现 Display trait
        impl Display for Point {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "I'm a point which x: {}, y: {}", self.x, self.y)
            }
        }

        // 实现 Drop trait
        // 析构函数
        impl Drop for Point {
            fn drop(&mut self) {
                println!("🆎 {} dropped!", self);
            }
        }

        impl Drop for Pot {
            fn drop(&mut self) {
                println!("🧿 {} dropped!", self);
            }
        }

        pub fn struct_methods_test() {
            let p = Point::new(-100, -200);
            {
                let mut p = Point::new(1, 2);
                p.move_by(1, 2);

                println!("{}", p);
            }
            // p.move_by(2, 4); // cannot mutate immutable variable `p`
            println!("{}", p);
        }
    }
}

// owership::struct_methods::struct_methods_test();
//
//
//
//  Point { x: Pot X: 2, y: Pot Y: 4 }
//  I'm a point which x: Pot X: 2, y: Pot Y: 4
//  🆎 I'm a point which x: Pot X: 2, y: Pot Y: 4 dropped!
//  🧿 Pot X: 2 dropped!
//  🧿 Pot Y: 4 dropped!
//  I'm a point which x: Pot X: 1, y: Pot Y: 2
//  🆎 I'm a point which x: Pot X: 1, y: Pot Y: 2 dropped!
//  🧿 Pot X: 1 dropped!
//  🧿 Pot Y: 2 dropped!
