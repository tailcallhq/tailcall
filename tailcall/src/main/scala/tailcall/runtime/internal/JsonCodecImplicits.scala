package tailcall.runtime.internal

import zio.json._

import java.net.{URI, URL}

object JsonCodecImplicits {
  implicit val urlCodec: JsonCodec[URL] = JsonCodec[String].transformOrFail[URL](
    string =>
      try Right(URI.create(string).toURL)
      catch { case _: Throwable => Left(s"Malformed url: ${string}") },
    _.toString,
  )
}
