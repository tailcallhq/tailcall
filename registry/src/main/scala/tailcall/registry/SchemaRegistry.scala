package tailcall.registry

import tailcall.runtime.ast.{Blueprint, Digest}
import zio._
import zio.rocksdb.RocksDB

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

  def persistent(path: String): ZLayer[Any, Throwable, SchemaRegistry] =
    RocksDB.live(path) >>> ZLayer.fromFunction(Persistence.apply _)

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
}
