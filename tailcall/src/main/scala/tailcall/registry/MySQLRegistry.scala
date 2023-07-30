package tailcall.registry

import com.mysql.cj.jdbc.MysqlDataSource
import io.getquill._
import io.getquill.jdbczio.Quill
import org.flywaydb.core.Flyway
import tailcall.registry.model.BlueprintSpec
import tailcall.runtime.model.{Blueprint, Digest}
import zio.{Task, ZIO, ZLayer}

import java.sql.Timestamp
import java.time.Instant
import java.util.Date

final case class MySQLRegistry(ctx: Quill[MySQLDialect, SnakeCase]) extends SchemaRegistry {

  import BlueprintSpec._
  import ctx._

  override def add(blueprint: Blueprint): Task[Digest] = {
    val blueprintSpec = BlueprintSpec(
      None,
      blueprint.digest.hex,
      blueprint.digest.alg,
      blueprint,
      Format.json,
      Option(Timestamp.from(Instant.now())),
      None,
    )

    ctx.transaction {
      for {
        // - find all the blueprints with the provided digest
        blueprints <- ctx.run(query[BlueprintSpec].filter(b =>
          b.digestHex == lift(blueprint.digest.hex) && b.digestAlg == lift(blueprint.digest.alg)
        ))

        // - if the list is empty then insert the new blueprint
        _          <- ctx.run(query[BlueprintSpec].insertValue(lift(blueprintSpec))).when(blueprints.isEmpty)

        // - if the list is non-empty then set the dropped to none, and created to current timestamp
        _ <- ctx.run {
          liftQuery(blueprints)
            .foreach(b => query[BlueprintSpec].filter(_.id == b.id).update(_.dropped -> lift(Option.empty[Timestamp])))
        }.when(blueprints.nonEmpty)
      } yield blueprint.digest
    }
  }

  override def drop(hex: String): Task[Boolean] = {
    val sql = quote(filterByDigest(hex).update(_.dropped -> lift(Option(new Timestamp(new Date().getTime)))))
    ctx.run(sql).map(_ > 0)
  }

  override def get(hex: String): Task[Option[Blueprint]] = {
    val sql = quote(filterByDigest(hex).map(_.blueprint))
    ctx.run(sql).map(_.headOption)
  }

  override def list(index: Int, max: Int): Task[List[Blueprint]] = {
    val sql = quote(query[BlueprintSpec].drop(lift(index)).take(lift(max)).map(_.blueprint))
    ctx.run(sql)
  }

  private def filterByDigest(hex: String): Quoted[EntityQuery[BlueprintSpec]] =
    quote(query[BlueprintSpec].filter(b => b.digestHex.like(lift(hex + "%")) && b.dropped.isEmpty))
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
