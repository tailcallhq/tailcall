use prost_reflect::prost_types::SourceCodeInfo;

pub struct CommentsBuilder {
    source_code_info: Option<SourceCodeInfo>,
}

impl CommentsBuilder {
    pub fn new(source_code_info: Option<SourceCodeInfo>) -> Self {
        Self { source_code_info }
    }

    pub fn get_comments(&self, path: &[i32]) -> Option<String> {
        self.source_code_info.as_ref().and_then(|info| {
            info.location
                .iter()
                .find(|loc| loc.path == path)
                .and_then(|loc| {
                    loc.leading_comments.as_ref().map(|c| {
                        c.lines()
                            .map(|line| line.trim_start_matches('*').trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join("\n  ")
                    })
                })
        })
    }
}
