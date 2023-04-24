package tailcall.runtime.internal

import zio.ZIO
import zio.http.Response

import java.nio.charset.StandardCharsets

object HttpAssertions {
  def assertStatusCodeIsAbove(code: Int, res: Response): ZIO[Any, Throwable, Unit] =
    if (res.status.code >= code) for {
      body <- res.body.asString(StandardCharsets.UTF_8)
      _    <- ZIO.logDebug(res.headers.mkString)
      _    <- ZIO.fail(new RuntimeException(s"HTTP Error: ${res.status.code}\n Response received: ${body}"))
    } yield res.body
    else ZIO.succeed(res.body)
}
