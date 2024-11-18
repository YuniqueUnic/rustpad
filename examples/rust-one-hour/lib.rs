const GLOBAL_CONST_VALUE: &'static str = "我是全局常量";
static GLOBAL_STATIC_VALUE: &'static str = "我是全局不可变静态变量";
static mut GLOBAL_STATIC_MUT_VALUE: &'static str = "我是全局可变静态变量";

fn consume_entity(e: String) {
    dbg!(e);
}

fn main() {}

mod clone_copy {

    // Clone and Copy
    fn clone_copy_relationship_0() {
        // Clone
        let clone_type: String = String::from("I'm a clone type which haven't impl the copy");
        let new_clone = clone_type;

        // dbg!(clone_type); // use of moved value: `clone_type`, value used here after move
        dbg!(new_clone);

        // Copy
        let copy_type: usize = 5;
        let new_copy = copy_type;

        dbg!(copy_type);
        dbg!(new_copy);
    }

    #[derive(Clone, Debug)]
    struct OnlyClone;

    fn clone_copy_relationship_1() {
        let only_clone = OnlyClone;
        // let new_only_clone = only_clone;
        // dbg!(only_clone);
    }

    #[derive(Copy, Clone, Debug)]
    struct CloneAndCopy;

    fn clone_copy_relationship_2() {
        let clone_and_copy = CloneAndCopy;
        let new_clone_and_copy = clone_and_copy;
        dbg!(clone_and_copy);
        dbg!(new_clone_and_copy);
    }
}
