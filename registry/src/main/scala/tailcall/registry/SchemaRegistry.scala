package tailcall.registry

import tailcall.runtime.model.{Blueprint, Digest}
import zio._

trait SchemaRegistry {
  def add(blueprint: Blueprint): Task[Digest]
  def drop(hex: String): Task[Boolean]
  def get(hex: String): Task[Option[Blueprint]]
  def list(index: Int, max: Int): Task[List[Blueprint]]
}

object SchemaRegistry {
  val PORT = 8080

  def add(blueprint: Blueprint): ZIO[SchemaRegistry, Throwable, Digest] =
    ZIO.serviceWithZIO[SchemaRegistry](_.add(blueprint))

  def digests(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Digest]] =
    list(index, max).flatMap(ZIO.foreach(_)(blueprint => ZIO.succeed(Digest.fromBlueprint(blueprint))))

  def drop(hex: String): ZIO[SchemaRegistry, Throwable, Boolean] = ZIO.serviceWithZIO[SchemaRegistry](_.drop(hex))

  def get(hex: String): ZIO[SchemaRegistry, Throwable, Option[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.get(hex))

  def list(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.list(index, max))

  def memory: ZLayer[Any, Throwable, SchemaRegistry] =
    ZLayer.fromZIO(for {
      ref <- Ref.make(List.empty[(String, Blueprint)])
      _   <- ZIO.log("Initialized in-memory schema registry")
    } yield MemoryRegistry(ref))

  def mysql(
    host: String,
    port: Int,
    uname: Option[String] = None,
    password: Option[String] = None,
    autoMigrate: Boolean = true,
  ): ZLayer[Any, Throwable, SchemaRegistry] = MySQLRegistry.default(host, port, uname, password, autoMigrate)

}
