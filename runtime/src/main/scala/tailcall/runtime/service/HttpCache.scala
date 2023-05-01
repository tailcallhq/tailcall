package tailcall.runtime.service
import tailcall.runtime.http.{HttpClient, Request}
import zio.cache.{Cache, CacheStats, Lookup}
import zio.http.Response
import zio.{Duration, Exit, Task, ZIO, ZLayer}

import java.text.SimpleDateFormat
import java.time.Instant
trait HttpCache  {
  def get(key: Request): Task[Response]
  def cacheStats: Task[CacheStats]
}
object HttpCache {
  def ttl(res: Response, currentMillis: => Instant = Instant.now()): Option[Duration] = {
    val headers      = res.headers.toList.map(x => String.valueOf(x.key).toLowerCase -> String.valueOf(x.value)).toMap
    val cacheControl = headers.get("cache-control").map(_.split(",").map(_.trim).toSet).getOrElse(Set.empty)
    val maxAge       = cacheControl.find(_.startsWith("max-age=")).map(_.split("=").last).flatMap(_.toLongOption)
    val expires      = maxAge.map(_ => None).getOrElse(headers.get("expires"))
    if (cacheControl.contains("private")) None
    else if (expires.isEmpty) maxAge.map(Duration.fromSeconds)
    else expires match {
      case Some(value) =>
        if (value matches "-1") { None }
        else {
          val date = new SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss z").parse(value).toInstant
          Option(Duration.fromInterval(currentMillis, date))
        }
      case None        => None
    }
  }

  final def default: ZLayer[Any, Throwable, HttpCache]                            = HttpClient.default >>> live
  final def live: ZLayer[HttpClient, Throwable, Live]                             = ZLayer(make.map(Live))
  final def make: ZIO[HttpClient, Throwable, Cache[Request, Throwable, Response]] =
    ZIO.service[HttpClient].flatMap(client => make(Lookup(client.request)))
  final def make(
    lookup: Lookup[Request, HttpClient, Throwable, Response]
  ): ZIO[HttpClient, Nothing, Cache[Request, Throwable, Response]] =
    for {
      cache <- Cache.makeWithKey(Int.MaxValue, lookup)(
        timeToLive = {
          case Exit.Success(value) => ttl(value) match {
              case Some(value) => value
              case None        => Duration.fromMillis(0)
            }
          case Exit.Failure(_)     => Duration.fromMillis(0)
        },
        keyBy = req => (req.method, req.url),
      )
    } yield cache
  final case class Live(cache: Cache[Request, Throwable, Response]) extends HttpCache {
    def get(key: Request): Task[Response] = cache.get(key)
    def cacheStats: Task[CacheStats]      = cache.cacheStats

  }
}
