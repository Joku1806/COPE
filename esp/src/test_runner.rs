#[cfg(test)]
use anyhow::Error;

// TODO: Look into replacing this with defmt-test. I already tried once, but
// couldn't get it to run unit tests.
#[cfg(test)]
pub fn run(tests: &[&(dyn Fn() -> Result<(), Error>)]) {
    esp_idf_svc::sys::link_patches(); //Needed for esp32-rs

    let mut failure_counter: usize = 0;
    for test in tests {
        match test() {
            Ok(_) => (),
            Err(error) => {
                failure_counter += 1;
                println!("Test failed: {}.", error);
            }
        }
    }

    println!(
        "Ran {} tests, {} passed and {} failed.",
        tests.len(),
        tests.len() - failure_counter,
        failure_counter
    );
}
