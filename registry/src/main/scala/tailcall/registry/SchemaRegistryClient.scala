package tailcall.registry

import tailcall.runtime.ast.{Blueprint, Digest}
import tailcall.runtime.service.EvaluationError
import zio.http._
import zio.schema.Schema
import zio.schema.codec.JsonCodec
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

    private def toBody(res: Response): ZIO[Any, Throwable, Body] =
      if (res.status.code >= 400) ZIO.fail(new RuntimeException(s"HTTP Error: ${res.status.code}"))
      else ZIO.succeed(res.body)

    private def buildURL(base: URL, path: String): ZIO[Any, RuntimeException, URL] = {
      ZIO.succeed(base.copy(path = base.path / path))
    }

    override def add(base: URL, blueprint: Blueprint): Task[Digest] =
      for {
        url          <- buildURL(base, "/schemas")
        response     <- client.request(Request.put(
          Body.fromChunk(Chunk.fromIterable(Blueprint.encode(blueprint).toString.getBytes(Charset.defaultCharset()))),
          url
        ))
        body         <- toBody(response)
        digestString <- body.asString
        digest       <- ZIO.fromEither(JsonCodec.jsonDecoder(Digest.schema).decodeJson(digestString))
          .mapError(EvaluationError.DecodingError(_))
      } yield digest

    override def get(base: URL, id: Digest): Task[Option[Blueprint]] =
      for {
        url       <- buildURL(base, s"/schemas/${id.hex}")
        response  <- client.request(Request.get(url))
        body      <- toBody(response)
        bpString  <- body.asString
        blueprint <- ZIO.fromEither(JsonCodec.jsonDecoder(Blueprint.schema).decodeJson(bpString))
          .mapError(EvaluationError.DecodingError(_))
      } yield Option(blueprint)

    override def list(base: URL, index: Int, max: Int): Task[List[Blueprint]] =
      for {
        url        <- buildURL(base, s"/schemas?index=${index}&max=${max}")
        response   <- client.request(Request.get(url))
        body       <- toBody(response)
        ls         <- body.asString
        blueprints <- ZIO.fromEither(JsonCodec.jsonDecoder(Schema[List[Blueprint]]).decodeJson(ls))
          .mapError(EvaluationError.DecodingError(_))
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
