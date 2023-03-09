package tailcall.server.service

import tailcall.runtime.ast.Blueprint
import tailcall.server.service.BinaryDigest.Digest
import zio.{Ref, Task, ZIO, ZLayer}

trait SchemaRegistry {
  def add(blueprint: Blueprint): Task[Unit]
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
    } yield new Memory(ref, bd))

  def add(blueprint: Blueprint): ZIO[SchemaRegistry, Throwable, Unit] =
    ZIO.serviceWithZIO[SchemaRegistry](_.add(blueprint))

  def get(id: Digest): ZIO[SchemaRegistry, Throwable, Option[Blueprint]] = ZIO.serviceWithZIO[SchemaRegistry](_.get(id))

  def list(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.list(index, max))

  def drop(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] = ZIO.serviceWithZIO[SchemaRegistry](_.drop(digest))

  def contains(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] =
    ZIO.serviceWithZIO[SchemaRegistry](_.contains(digest))

  final case class Memory(ref: Ref[Map[Digest, Blueprint]], bd: BinaryDigest) extends SchemaRegistry {

    override def add(blueprint: Blueprint): Task[Unit] = ???

    override def get(id: Digest): Task[Option[Blueprint]] = ???

    override def list(index: Int, max: Int): Task[List[Blueprint]] = ???

    override def drop(digest: Digest): Task[Boolean] = ???

    override def contains(digest: Digest): Task[Boolean] = ???
  }
}
