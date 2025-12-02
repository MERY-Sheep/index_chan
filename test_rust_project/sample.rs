// Test Rust file for index-chan

pub fn used_function() {
    println!("This function is used");
    helper_function();
}

fn helper_function() {
    println!("Helper function");
}

fn unused_function() {
    println!("This function is never called");
}

pub fn another_used() {
    used_function();
}

fn dead_code_example() {
    // This is dead code
    let x = 42;
}
