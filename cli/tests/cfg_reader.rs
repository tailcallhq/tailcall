#[cfg(test)]
mod reader_tests {
    use std::path::PathBuf;

    use cli::{init_file, init_http};
    use corex::config::reader::ConfigReader;
    use corex::config::{Config, Type, Upstream};
    use tokio::io::AsyncReadExt;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_all() {
        let mut cfg = Config::default();
        cfg.schema.query = Some("Test".to_string());
        cfg = cfg.types([("Test", Type::default())].to_vec());

        let server = start_mock_server();
        let header_serv = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/bar.graphql");
            then.status(200).body(cfg.to_sdl());
        });

        let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        examples_dir.pop();
        examples_dir.push("examples");

        let mut phj = examples_dir.clone();
        phj.push("jsonplaceholder.json");

        let mut json = String::new();
        tokio::fs::File::open(phj)
            .await
            .unwrap()
            .read_to_string(&mut json)
            .await
            .unwrap();

        let foo_json_server = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/foo.json");
            then.status(200).body(json);
        });

        let port = server.port();
        let mut phy = examples_dir.clone();
        phy.push("jsonplaceholder.yml");

        let files: Vec<String> = [
            phy.to_str().unwrap(), // config from local file
            format!("http://localhost:{port}/bar.graphql").as_str(), // with content-type header
            format!("http://localhost:{port}/foo.json").as_str(), // with url extension
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(init_file(), init_http(&Upstream::default()));
        let c = cr.read(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "Test", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
        foo_json_server.assert(); // checks if the request was actually made
        header_serv.assert();
    }

    #[tokio::test]
    async fn test_local_files() {
        let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        examples_dir.pop();
        examples_dir.push("examples");
        let mut phj = examples_dir.clone();
        phj.push("jsonplaceholder.json");
        let mut phy = examples_dir.clone();
        phy.push("jsonplaceholder.yml");
        let mut phg = examples_dir.clone();
        phg.push("jsonplaceholder.graphql");

        let files: Vec<String> = [
            phj.to_str().unwrap(),
            phy.to_str().unwrap(),
            phg.to_str().unwrap(),
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(init_file(), init_http(&Upstream::default()));
        let c = cr.read(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
    }
}
