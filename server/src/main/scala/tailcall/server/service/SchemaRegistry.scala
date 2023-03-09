package tailcall.server.service

import tailcall.runtime.ast.Blueprint
import tailcall.server.service.BinaryDigest.Digest
import zio.{Ref, Task, UIO, ZIO, ZLayer}

trait SchemaRegistry {
  def add(blueprint: Blueprint): Task[Digest]
  def get(id: Digest): Task[Option[Blueprint]]
  def list(index: Int, max: Int): Task[List[Blueprint]]
  def drop(digest: Digest): Task[Boolean]
  def contains(digest: Digest): Task[Boolean]
}

object SchemaRegistry {
  def memory: ZLayer[BinaryDigest, Nothing, SchemaRegistry] =
    ZLayer.fromZIO(for {
      ref <- Ref.make(Map.empty[Digest, Blueprint])
      bd  <- ZIO.service[BinaryDigest]
    } yield Memory(ref, bd))

  def add(blueprint: Blueprint): ZIO[SchemaRegistry, Throwable, Digest] =
    ZIO.serviceWithZIO[SchemaRegistry](_.add(blueprint))

  def get(id: Digest): ZIO[SchemaRegistry, Throwable, Option[Blueprint]] = ZIO.serviceWithZIO[SchemaRegistry](_.get(id))

  def list(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.list(index, max))

  def drop(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] = ZIO.serviceWithZIO[SchemaRegistry](_.drop(digest))

  def contains(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] =
    ZIO.serviceWithZIO[SchemaRegistry](_.contains(digest))

  final case class Memory(ref: Ref[Map[Digest, Blueprint]], bd: BinaryDigest) extends SchemaRegistry {

    override def add(blueprint: Blueprint): Task[Digest] = {
      val digest: Digest = bd.digest(blueprint)
      ref.update(_.+(digest -> blueprint)).as(digest)
    }

    override def get(id: Digest): Task[Option[Blueprint]] = ref.get.map(_.get(id))

    override def list(index: Int, max: Int): Task[List[Blueprint]] = ref.get.map(_.values.toList)

    override def drop(digest: Digest): UIO[Boolean] =
      ref.modify(map => if (map.contains(digest)) (true, map - digest) else (false, map))

    override def contains(digest: Digest): Task[Boolean] = ref.get.map(_.contains(digest))
  }
}
