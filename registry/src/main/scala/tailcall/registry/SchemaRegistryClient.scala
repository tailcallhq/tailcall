package tailcall.registry

import tailcall.runtime.internal.HttpAssertions
import tailcall.runtime.model.{Blueprint, Digest}
import zio.http._
import zio.http.model.Status
import zio.json.DecoderOps
import zio.{Chunk, Task, ZIO, ZLayer}

import java.nio.charset.Charset

trait SchemaRegistryClient {
  def add(base: URL, blueprint: Blueprint): Task[Digest]
  def get(base: URL, id: Digest): Task[Option[Blueprint]]
  def list(base: URL, index: Int, max: Int): Task[List[Blueprint]]
  def drop(base: URL, digest: Digest): Task[Boolean]
}

object SchemaRegistryClient {
  final case class Live(client: Client) extends SchemaRegistryClient {

    private def buildURL(base: URL, path: String): ZIO[Any, RuntimeException, URL] = {
      ZIO.succeed(base.copy(path = base.path / path))
    }

    override def add(base: URL, blueprint: Blueprint): Task[Digest] =
      for {
        url          <- buildURL(base, "/schemas")
        response     <- client.request(Request.put(
          Body.fromChunk(Chunk.fromIterable(Blueprint.encode(blueprint).toString.getBytes(Charset.defaultCharset()))),
          url,
        ))
        body         <- HttpAssertions.assertStatusCodeIsAbove(400, response)
        digestString <- body.asString
        digest       <- ZIO.fromEither(digestString.fromJson[Digest]).mapError(new RuntimeException(_))
      } yield digest

    override def get(base: URL, id: Digest): Task[Option[Blueprint]] =
      for {
        url      <- buildURL(base, s"/schemas/${id.hex}")
        response <- client.request(Request.get(url))
        maybe    <- response.status match {
          case Status.NotFound => ZIO.succeed(None)
          case _               => for {
              body      <- HttpAssertions.assertStatusCodeIsAbove(400, response)
              bpString  <- body.asString
              blueprint <- ZIO.fromEither(bpString.fromJson[Blueprint]).mapError(new RuntimeException(_))
            } yield Option(blueprint)
        }
      } yield maybe

    override def list(base: URL, index: Int, max: Int): Task[List[Blueprint]] =
      for {
        url        <- buildURL(base, s"/schemas?index=${index}&max=${max}")
        response   <- client.request(Request.get(url))
        body       <- HttpAssertions.assertStatusCodeIsAbove(400, response)
        ls         <- body.asString
        blueprints <- ZIO.fromEither(ls.fromJson[List[Blueprint]]).mapError(new RuntimeException(_))
      } yield blueprints

    override def drop(base: URL, digest: Digest): Task[Boolean] =
      for {
        url      <- buildURL(base, s"/schemas/${digest.hex}")
        response <- client.request(Request.delete(url))
        out      <-
          if (response.status.code >= 400) ZIO.fail(new RuntimeException(s"HTTP Error: ${response.status.code}"))
          else ZIO.succeed(response.status.code == 200)
      } yield out
  }

  def live: ZLayer[Client, Nothing, SchemaRegistryClient]   = ZLayer.fromFunction(Live.apply _)
  def default: ZLayer[Any, Throwable, SchemaRegistryClient] = Client.default >>> live
}
