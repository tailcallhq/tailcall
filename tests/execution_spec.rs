mod executionspec;

use std::path::Path;

use executionspec::spec::load_and_test_execution_spec;

fn run_execution_spec(path: &Path) -> datatest_stable::Result<()> {
    let result = tokio_test::block_on(load_and_test_execution_spec(path));

    Ok(result?)
}

datatest_stable::harness!(run_execution_spec, "tests/execution", r"^.*\.md$");
