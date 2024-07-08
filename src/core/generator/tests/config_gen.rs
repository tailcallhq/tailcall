mod json_to_config_spec;
mod open_api_to_config_spec;

datatest_stable::harness!(
    json_to_config_spec::run_json_to_config_spec,
    "src/core/generator/tests/fixtures/json",
    r"^.*\.json",
    open_api_to_config_spec::run_open_api_to_config_spec,
    "src/core/generator/tests/fixtures/openapi",
    r"^.*\.yml"
);
