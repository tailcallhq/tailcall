package tailcall.registry

import com.mysql.cj.jdbc.MysqlDataSource
import io.getquill._
import io.getquill.context.ZioJdbc.QuillZioDataSourceExt
import io.getquill.context.qzio.ImplicitSyntax
import tailcall.registry.model.BlueprintSpec
import tailcall.runtime.model.{Blueprint, Digest}
import zio.{Task, ZIO}

import java.sql.Timestamp
import java.util.Date
import javax.sql.DataSource
final case class MySQLRegistry(source: javax.sql.DataSource, ctx: MysqlZioJdbcContext[SnakeCase])
    extends SchemaRegistry {

  import BlueprintSpec._

  implicit private val dataSource: ImplicitSyntax.Implicit[DataSource] = ImplicitSyntax.Implicit(source)
  import ctx._

  override def add(blueprint: Blueprint): Task[Digest] = {
    val sql = quote(query[BlueprintSpec].insert(
      _.digestHex       -> lift(blueprint.digest.hex),
      _.digestAlg       -> lift(blueprint.digest.alg),
      _.blueprint       -> lift(blueprint),
      _.blueprintFormat -> lift(Format.Json: Format),
    ))
    ctx.run(sql).as(blueprint.digest).implicitDS
  }

  override def drop(digest: Digest): Task[Boolean] = {
    val sql = quote(filterByDigest(digest).update(_.dropped -> lift(Option(new Timestamp(new Date().getTime)))))
    ctx.run(sql).map(_ > 0).implicitDS
  }

  override def get(digest: Digest): Task[Option[Blueprint]] = {
    val sql = quote(filterByDigest(digest).map(_.blueprint))
    ctx.run(sql).map(_.headOption).implicitDS
  }

  override def list(index: Int, max: Int): Task[List[Blueprint]] = {
    val sql = quote(query[BlueprintSpec].drop(lift(index)).take(lift(max)).map(_.blueprint))
    ctx.run(sql).implicitDS
  }

  private def filterByDigest(digest: Digest): Quoted[EntityQuery[BlueprintSpec]] =
    quote(query[BlueprintSpec].filter(b => b.digestHex == lift(digest.hex) && b.digestAlg == lift(digest.alg)))
}

object MySQLRegistry {
  def dataSource(
    host: String,
    port: Int,
    uname: Option[String],
    pass: Option[String],
  ): ZIO[Any, Throwable, MysqlDataSource] =
    for {
      dataSource <- ZIO.attempt(new MysqlDataSource())
      _          <- ZIO.attempt {
        dataSource.setServerName(host)
        dataSource.setPort(port)
        dataSource.setDatabaseName("tailcall_main_db")
        uname.foreach(dataSource.setUser)
        pass.foreach(dataSource.setPassword)
        dataSource.setCreateDatabaseIfNotExist(true)
      }
    } yield dataSource
}
