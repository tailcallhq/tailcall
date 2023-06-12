package tailcall.cli

import zio.http.URL

object CLIConfig {
  val remote: URL = URL.fromString("https://cloud.tailcall.run").getOrElse(null)
}
