// Test Rust file for index-chan

pub fn used_function() {
    println!("This function is used");
    helper_function();
}

fn helper_function() {
    println!("Helper function");
}

#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
fn unused_function() {
    println!("This function is never called");
}

#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
pub fn another_used() {
    used_function();
}
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests
#[allow(dead_code)] // index-chan: Test file - may be used in tests

fn dead_code_example() {
    // This is dead code
    let x = 42;
}