package tailcall.registry

import com.mysql.cj.jdbc.MysqlDataSource
import io.getquill._
import io.getquill.jdbczio.Quill
import org.flywaydb.core.Flyway
import tailcall.registry.model.BlueprintSpec
import tailcall.runtime.model.{Blueprint, Digest}
import zio.{Task, ZIO, ZLayer}

import java.sql.Timestamp
import java.util.Date

final case class MySQLRegistry(ctx: Quill[MySQLDialect, SnakeCase]) extends SchemaRegistry {

  import BlueprintSpec._
  import ctx._

  override def add(blueprint: Blueprint): Task[Digest] = {
    val sql = quote(query[BlueprintSpec].insert(
      _.digestHex       -> lift(blueprint.digest.hex),
      _.digestAlg       -> lift(blueprint.digest.alg),
      _.blueprint       -> lift(blueprint),
      _.blueprintFormat -> lift(Format.Json: Format),
    ))
    ctx.run(sql).as(blueprint.digest)
  }

  override def drop(digest: Digest): Task[Boolean] = {
    val sql = quote(filterByDigest(digest).update(_.dropped -> lift(Option(new Timestamp(new Date().getTime)))))
    ctx.run(sql).map(_ > 0)
  }

  override def get(digest: Digest): Task[Option[Blueprint]] = {
    val sql = quote(filterByDigest(digest).map(_.blueprint))
    ctx.run(sql).map(_.headOption)
  }

  override def list(index: Int, max: Int): Task[List[Blueprint]] = {
    val sql = quote(query[BlueprintSpec].drop(lift(index)).take(lift(max)).map(_.blueprint))
    ctx.run(sql)
  }

  private def filterByDigest(digest: Digest): Quoted[EntityQuery[BlueprintSpec]] =
    quote(query[BlueprintSpec].filter(b => b.digestHex == lift(digest.hex) && b.digestAlg == lift(digest.alg)))
}

object MySQLRegistry {
  def default(
    host: String,
    port: Int,
    uname: Option[String],
    pass: Option[String],
    autoMigrate: Boolean,
  ): ZLayer[Any, Throwable, MySQLRegistry] =
    (dataSource(host, port, uname, pass, autoMigrate) >>> Quill.Mysql.fromNamingStrategy(SnakeCase)) >>> live

  def live: ZLayer[Quill[MySQLDialect, SnakeCase], Nothing, MySQLRegistry] =
    ZLayer.fromFunction((mysql: Quill[MySQLDialect, SnakeCase]) => MySQLRegistry(mysql))

  private def dataSource(
    host: String,
    port: Int,
    uname: Option[String],
    pass: Option[String],
    autoMigrate: Boolean,
  ): ZLayer[Any, Throwable, MysqlDataSource] =
    ZLayer.fromZIO {
      for {
        _          <- ZIO.log(s"Initialized persistent datasource @${host}:${port}")
        dataSource <- ZIO.attempt(new MysqlDataSource())
        _          <- ZIO.attempt {
          dataSource.setServerName(host)
          dataSource.setPort(port)
          dataSource.setDatabaseName("tailcall_main_db")
          uname.foreach(dataSource.setUser)
          pass.foreach(dataSource.setPassword)
          dataSource.setCreateDatabaseIfNotExist(true)

          Quill.Mysql
        }
        _          <- migrate(dataSource).when(autoMigrate)
      } yield dataSource
    }

  private def migrate(dataSource: MysqlDataSource): ZIO[Any, Throwable, Unit] = {
    for {
      flyway    <- ZIO.succeed(Flyway.configure().dataSource(dataSource).load())
      migration <- ZIO.attemptBlocking(flyway.migrate())
      _         <- ZIO.log(s"Migrations executed: ${migration.migrationsExecuted}")
    } yield ()
  }
}
