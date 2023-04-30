package tailcall.runtime.service
import tailcall.runtime.http.{HttpClient, Request}
import zio.cache.{Cache, CacheStats, Lookup}
import zio.http.Response
import zio.{Duration, Exit, Task, ZIO, ZLayer}
trait HttpCache  {
  def get(key: Request): Task[Response]
  def cacheStats: Task[CacheStats]
}
object HttpCache {
  def ttl(res: Response): Option[Duration]                                        =
    for {
      cacheControl <- res.headers.get("Cache-Control").map(_.split(",").map(_.trim).toSet)
      maxAge  = cacheControl.find(_.startsWith("max-age=")).flatMap(_.split("=").last.toLongOption)
      expires = { if (maxAge.nonEmpty) None else res.headers.get("Expires") }
      duration <-
        if (cacheControl.contains("private")) Some(Duration.fromMillis(0))
        else if (expires.isEmpty) maxAge.map(Duration.fromSeconds)
        else expires.map(ts =>
          if (!ts.toIntOption.contains(-1)) { Duration.fromInstant(java.time.Instant.parse(ts)) }
          else { Duration.fromMillis(0) }
        )
    } yield duration
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
