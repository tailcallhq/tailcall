package tailcall.registry

import tailcall.runtime.ast.{Blueprint, Digest}
import tailcall.runtime.http.{HttpClient, Method, Request}
import tailcall.runtime.service.EvaluationError
import zio._
import zio.http.{Body, Response}
import zio.rocksdb.RocksDB
import zio.schema.Schema
import zio.schema.codec.JsonCodec

import java.nio.charset.Charset
import java.nio.file.Files

trait SchemaRegistry {
  def add(blueprint: Blueprint): Task[Digest]
  def get(id: Digest): Task[Option[Blueprint]]
  def list(index: Int, max: Int): Task[List[Blueprint]]
  def drop(digest: Digest): Task[Boolean]
}

object SchemaRegistry {
  val PORT = 8080

  def memory: ZLayer[Any, Nothing, SchemaRegistry] =
    ZLayer.fromZIO(for { ref <- Ref.make(Map.empty[Digest, Blueprint]) } yield Memory(ref))

  def persistent: ZLayer[Any, Throwable, SchemaRegistry] =
    RocksDB.live(Files.createTempDirectory("rocksDB-").toFile.getAbsolutePath) >>> ZLayer
      .fromFunction(Persistence.apply _)

  def client: ZLayer[HttpClient with String, Nothing, SchemaRegistry] = ZLayer.fromFunction(Client.apply _)

  def add(blueprint: Blueprint): ZIO[SchemaRegistry, Throwable, Digest] =
    ZIO.serviceWithZIO[SchemaRegistry](_.add(blueprint))

  def get(id: Digest): ZIO[SchemaRegistry, Throwable, Option[Blueprint]] = ZIO.serviceWithZIO[SchemaRegistry](_.get(id))

  def list(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.list(index, max))

  def digests(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Digest]] =
    list(index, max).flatMap(ZIO.foreach(_)(blueprint => ZIO.succeed(Digest.fromBlueprint(blueprint))))

  def drop(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] = ZIO.serviceWithZIO[SchemaRegistry](_.drop(digest))

  private def decode(bytes: Array[Byte]): Task[Blueprint] = {
    Blueprint.decode(new String(bytes)) match {
      case Left(value)  => ZIO.fail(new RuntimeException(value))
      case Right(value) => ZIO.succeed(value)
    }
  }

  final case class Memory(ref: Ref[Map[Digest, Blueprint]]) extends SchemaRegistry {

    override def add(blueprint: Blueprint): Task[Digest] = {
      val digest: Digest = blueprint.digest
      ref.update(_.+(digest -> blueprint)).as(digest)
    }

    override def get(id: Digest): Task[Option[Blueprint]] = ref.get.map(_.get(id))

    override def list(index: Int, max: Int): Task[List[Blueprint]] = ref.get.map(_.values.toList)

    override def drop(digest: Digest): UIO[Boolean] =
      ref.modify(map => if (map.contains(digest)) (true, map - digest) else (false, map))
  }

  final case class Persistence(db: RocksDB) extends SchemaRegistry {
    override def add(blueprint: Blueprint): Task[Digest] = {
      val digest = blueprint.digest
      val value  = Blueprint.encode(blueprint).toString.getBytes()

      db.put(digest.getBytes, value).as(digest)
    }

    override def get(digest: Digest): Task[Option[Blueprint]] = {
      for {
        option    <- db.get(digest.getBytes)
        blueprint <- option match {
          case Some(bytes) => decode(bytes).map(Option(_))
          case None        => ZIO.succeed(None)
        }
      } yield blueprint
    }

    override def list(index: Int, max: Int): Task[List[Blueprint]] = {
      for {
        chunk      <- db.newIterator.take(max).runCollect
        blueprints <- ZIO.foreach(chunk) { case (_, value) => decode(value) }
      } yield blueprints.toList
    }

    override def drop(digest: Digest): Task[Boolean] = {
      for {
        contains <- db.get(digest.getBytes).map(_.isEmpty)
        _        <- db.delete(digest.getBytes).when(contains)
      } yield !contains
    }
  }

  private def toBody(res: Response): ZIO[Any, Throwable, Body] =
    if (res.status.code >= 400) ZIO.fail(new RuntimeException(s"HTTP Error: ${res.status.code}"))
    else ZIO.succeed(res.body)
  final case class Client(host: String, http: HttpClient) extends SchemaRegistry {
    override def add(blueprint: Blueprint): Task[Digest]  =
      for {
        response     <- http.request(Request(
          host + "/schemas",
          Method.PUT,
          body = Chunk.fromIterable(Blueprint.encode(blueprint).toString.getBytes(Charset.defaultCharset()))
        ))
        body         <- toBody(response)
        digestString <- body.asString
        digest       <- ZIO.fromEither(JsonCodec.jsonDecoder(Digest.schema).decodeJson(digestString))
          .mapError(EvaluationError.DecodingError(_))
      } yield digest
    override def get(id: Digest): Task[Option[Blueprint]] =
      for {
        response  <- http.request(Request(s"${host}/schemas/${id.alg.name}/${id.hex}"))
        body      <- toBody(response)
        bpString  <- body.asString
        blueprint <- ZIO.fromEither(JsonCodec.jsonDecoder(Blueprint.schema).decodeJson(bpString))
          .mapError(EvaluationError.DecodingError(_))
      } yield Option(blueprint)

    override def list(index: Int, max: Int): Task[List[Blueprint]] =
      for {
        response   <- http.request(Request(host + "/schemas"))
        body       <- toBody(response)
        ls         <- body.asString
        blueprints <- ZIO.fromEither(JsonCodec.jsonDecoder(Schema[List[Blueprint]]).decodeJson(ls))
          .mapError(EvaluationError.DecodingError(_))
      } yield blueprints

    override def drop(digest: Digest): Task[Boolean] =
      for {
        response <- http.request(Request(s"${host}/schemas/${digest.alg.name}/${digest.hex}", Method.DELETE))
        out      <-
          if (response.status.code >= 400) ZIO.fail(new RuntimeException(s"HTTP Error: ${response.status.code}"))
          else ZIO.succeed(response.status.code == 200)
      } yield out
  }
}
