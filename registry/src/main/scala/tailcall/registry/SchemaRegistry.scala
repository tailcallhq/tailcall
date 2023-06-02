package tailcall.registry

import com.mysql.cj.jdbc.MysqlDataSource
import io.getquill._
import org.flywaydb.core.Flyway
import tailcall.runtime.model.{Blueprint, Digest}
import zio._
import zio.redis.Redis

trait SchemaRegistry {
  def add(blueprint: Blueprint): Task[Digest]
  def drop(digest: Digest): Task[Boolean]
  def get(id: Digest): Task[Option[Blueprint]]
  def list(index: Int, max: Int): Task[List[Blueprint]]
}

object SchemaRegistry {
  val PORT = 8080

  def add(blueprint: Blueprint): ZIO[SchemaRegistry, Throwable, Digest] =
    ZIO.serviceWithZIO[SchemaRegistry](_.add(blueprint))

  def digests(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Digest]] =
    list(index, max).flatMap(ZIO.foreach(_)(blueprint => ZIO.succeed(Digest.fromBlueprint(blueprint))))

  def drop(digest: Digest): ZIO[SchemaRegistry, Throwable, Boolean] = ZIO.serviceWithZIO[SchemaRegistry](_.drop(digest))

  def get(id: Digest): ZIO[SchemaRegistry, Throwable, Option[Blueprint]] = ZIO.serviceWithZIO[SchemaRegistry](_.get(id))

  def list(index: Int, max: Int): ZIO[SchemaRegistry, Throwable, List[Blueprint]] =
    ZIO.serviceWithZIO[SchemaRegistry](_.list(index, max))

  def memory: ZLayer[Any, Throwable, SchemaRegistry] =
    ZLayer.fromZIO(for {
      ref <- Ref.make(Map.empty[Digest, Blueprint])
      _   <- ZIO.log("Initialized in-memory schema registry")
    } yield MemoryRegistry(ref))

  def mysql(
    host: String,
    port: Int,
    uname: Option[String],
    password: Option[String],
    autoMigrate: Boolean = true,
  ): ZLayer[Any, Throwable, SchemaRegistry] =
    ZLayer.fromZIO {
      for {
        _          <- ZIO.log(s"Initialized persistent schema registry @${host}:${port}")
        dataSource <- MySQLRegistry.dataSource(host, port, uname, password)
        _          <- migrate(dataSource).when(autoMigrate)
      } yield MySQLRegistry(dataSource, new MysqlZioJdbcContext(SnakeCase))
    }

  def redis: ZLayer[Redis, Nothing, SchemaRegistry] = ZLayer.fromFunction(RedisRegistry(_))

  private def migrate(dataSource: MysqlDataSource): ZIO[Any, Throwable, Unit] = {
    for {
      flyway    <- ZIO.succeed(Flyway.configure().dataSource(dataSource).load())
      migration <- ZIO.attemptBlocking(flyway.migrate())
      _         <- ZIO.log(s"Migrations executed: ${migration.migrationsExecuted}")
    } yield ()
  }
}
