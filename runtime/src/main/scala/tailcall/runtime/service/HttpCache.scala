package tailcall.runtime.service
import tailcall.runtime.http.Request
import zio._
import zio.cache.{Cache, CacheStats, Lookup}
import zio.http.Response
import zio.http.model.Headers

import java.text.SimpleDateFormat
import java.time.Instant
private[tailcall] trait HttpCache  {
  def get(key: Request): Task[Response]
  def init(lookupFn: Lookup[Request, Any, Throwable, Response]): UIO[Unit]
  def cacheStats: Task[CacheStats]
}
private[tailcall] object HttpCache {
  val dateFormat = new SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss z")
  final def ttl(res: Response, currentMillis: => Instant = Instant.now()): Option[Duration] =
    ttlHeaders(res.headers, currentMillis)

  final def ttlHeaders(headers: Headers, currentMillis: => Instant = Instant.now()): Option[Duration] = {
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

  def live(cacheSize: Int): ZLayer[Any, Nothing, Live] =
    ZLayer.fromZIO(for {
      cache <- Ref.make[Option[Cache[Request, Throwable, Response]]](None)
    } yield Live(cache, cacheSize))

  final case class Live(cache: Ref[Option[Cache[Request, Throwable, Response]]], cacheSize: Int) extends HttpCache {
    def init(lookupFn: Lookup[Request, Any, Throwable, Response]): UIO[Unit] = {
      Cache.makeWithKey(cacheSize, lookupFn)(
        timeToLive = {
          case Exit.Success(value) => ttl(value) match {
              case Some(value) => value
              case None        => Duration.fromMillis(0)
            }
          case Exit.Failure(_)     => Duration.fromMillis(0)
        },
        keyBy = req => (req.method, req.url),
      ).flatMap(x => cache.set(Option(x)))
    }
    def get(key: Request): IO[Throwable, Response]                           =
      cache.get.flatMap {
        case Some(value) => value.get(key)
        case None        => ZIO.fail(new IllegalStateException("Cache not initialized"))
      }
    def cacheStats: Task[CacheStats]                                         =
      cache.get.flatMap {
        case Some(value) => value.cacheStats
        case None        => ZIO.fail(new IllegalStateException("Cache not initialized"))
      }
  }
}
