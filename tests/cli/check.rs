pub mod test {
    use std::io::Cursor;

    use tailcall::cli::tc::check::{check_command, CheckParams};
    use tailcall::core::blueprint::Blueprint;
    use tokio::runtime::Runtime;

    pub fn run_check_command_spec(path: &std::path::Path) -> datatest_stable::Result<()> {
        let path = path.to_path_buf();
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async move {
            run_test(&path.to_string_lossy()).await?;
            Ok(())
        })
    }

    async fn run_test(path: &str) -> anyhow::Result<()> {
        let runtime = tailcall::cli::runtime::init(&Blueprint::default());
        let config_reader = tailcall::core::config::reader::ConfigReader::init(runtime.clone());
        let params = CheckParams {
            file_paths: vec![path.to_string()],
            n_plus_one_queries: false,
            schema: false,
            format: None,
            runtime: runtime.clone(),
        };

        let mut output_buffer = Cursor::new(Vec::new());

        check_command(params, &config_reader, Some(&mut output_buffer)).await?;

        let output_string = String::from_utf8(output_buffer.into_inner())?;
        insta::assert_snapshot!(path, output_string);

        Ok(())
    }

    pub fn check_command_test(path: &std::path::Path) -> datatest_stable::Result<()> {
        let path = path.to_path_buf();
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async move {
            run_test(&path.to_string_lossy()).await?;
            Ok(())
        })
    }
}

datatest_stable::harness!(
    test::check_command_test,
    "tests/cli/fixtures/check",
    r"^.*\..*$",
);
