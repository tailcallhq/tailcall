mod core;

use core::spec::load_and_test_execution_spec;
use std::path::Path;

fn run_execution_spec(path: &Path) -> datatest_stable::Result<()> {
    let result = tokio_test::block_on(load_and_test_execution_spec(path));

    Ok(result?)
}

datatest_stable::harness!(run_execution_spec, "tests/execution", r"^.*\.md$");
