package tailcall.runtime.service
import zio._
import zio.http.Response
import zio.http.model.Headers

import java.text.SimpleDateFormat
import java.time.Instant

private[tailcall] object HttpCache {
  private[tailcall] val dateFormat = new SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss z")
  final def ttl(res: Response, currentMillis: => Instant = Instant.now()): Option[Duration] =
    ttlHeaders(res.headers, currentMillis)

  final def ttlHeaders(headers: Headers, currentMillis: Instant = Instant.now()): Option[Duration] = {
    val headerList   = headers.toList.map(x => String.valueOf(x.key).toLowerCase -> String.valueOf(x.value)).toMap
    val cacheControl = headerList.get("cache-control").map(_.split(",").map(_.trim).toSet).getOrElse(Set.empty)
    val maxAge       = cacheControl.find(_.startsWith("max-age=")).map(_.split("=").last).flatMap(_.toLongOption)
    val expires      = maxAge.map(_ => None).getOrElse(headerList.get("expires"))
    if (cacheControl.contains("private")) None
    else if (expires.isEmpty) maxAge.map(Duration.fromSeconds)
    else expires match {
      case Some(value) =>
        if (value matches "-1") { None }
        else {
          val date = dateFormat.parse(value).toInstant
          Option(Duration.fromInterval(currentMillis, date))
        }
      case None        => None
    }
  }
}
