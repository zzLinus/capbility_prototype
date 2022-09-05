#![cfg(kernel_test)]

use alloc::string::*;

pub struct TestResult {
    pub passed: u32,
    pub failed: u32,
}
struct TestlistElem (fn() -> TestResult, String);

pub fn test_main() {
    let list_of_test = [
        // Add your general test function here. In the form of:
        // TestlistElem(your_test_name, String::from("your_test_name")),
        TestlistElem(test_framework_default, String::from("test_framework_default")),
    ];

    println!("Start testing.");
    
    let mut passed_count = 0;
    let mut failed_count = 0;
    
    for test in list_of_test {
        if test.1 == String::from("test_framework_default") {
            continue;
        }
        println!("Running \"{}\"", test.1);
        let test_result = test.0();
        passed_count += test_result.passed;
        failed_count += test_result.failed;

        print!("Test \"{}\" finished --- ", test.1);
        if test_result.failed == 0 {
            println!("\x1b[32mall {} test(s) passed.\x1b[0m", test_result.passed + test_result.failed);
        } else {
            if test_result.passed == 0 {
                println!("\x1b[31mall {} test(s) failed.\x1b[0m", test_result.failed);
            } else {
                println!("\x1b[31m{} test(s) failed.\x1b[0m", test_result.failed);
            }
        }
    }

    println!();
    if failed_count == 0 {
        println!("All test finished, \x1b[32mall {} test(s) passed.\x1b[0m", passed_count);
    } else {
        if passed_count == 0 {
            println!("All test finished, \x1b[31mall {} test(s) failed.\x1b[0m", failed_count);
        } else {
            println!("All test finished, \x1b[31m{} test(s) failed.\x1b[0m", failed_count);
        }
    }
}

fn test_framework_default() -> TestResult {
    TestResult { passed: 0, failed: 0 }
}