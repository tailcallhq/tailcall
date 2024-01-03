use async_graphql::dynamic::Schema;
use async_graphql::SDLExportOptions;

/// SDL returned from AsyncSchemaInner isn't standard
/// We clean it up before returning.
pub fn print_schema(schema: Schema) -> String {
  let sdl = schema.sdl_with_options(SDLExportOptions::new().sorted_fields());
  let mut result = String::new();
  let mut prev_line_empty = false;

  for line in sdl.lines() {
    let trimmed_line = line.trim();

    if trimmed_line.is_empty() {
      if !prev_line_empty {
        result.push('\n');
      }
      prev_line_empty = true;
    } else {
      let formatted_line = if line.starts_with('\t') {
        line.replace('\t', "  ")
      } else {
        line.to_string()
      };
      result.push_str(&formatted_line);
      result.push('\n');
      prev_line_empty = false;
    }
  }

  result.trim().to_string()
}
