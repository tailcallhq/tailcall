package tailcall.registry

import com.mysql.cj.jdbc.MysqlDataSource
import io.getquill._
import tailcall.runtime.model.{Blueprint, Digest}
import zio._
import zio.json.{DecoderOps, EncoderOps}
import zio.redis.Redis

import java.sql.Timestamp
import java.util.Date

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
    } yield Memory(ref))

  def mysql(
    host: String,
    port: Int,
    uname: Option[String],
    password: Option[String],
  ): ZLayer[Any, Throwable, SchemaRegistry] =
    ZLayer.fromZIO {
      for {
        dataSource <- ZIO.attempt(new MysqlDataSource())
        _          <- ZIO.attempt {
          dataSource.setServerName(host)
          dataSource.setPort(port)
          dataSource.setDatabaseName("tailcall_main_db")
          uname.foreach(dataSource.setUser)
          password.foreach(dataSource.setPassword)
        }
        _          <- ZIO.log(s"Initialized persistent schema registry @${host}:${port}")
      } yield FromMySQL(dataSource, new MysqlZioJdbcContext(SnakeCase))
    }

  def redis: ZLayer[Redis, Nothing, SchemaRegistry] = ZLayer.fromFunction(FromRedis(_))

  final case class Memory(ref: Ref[Map[Digest, Blueprint]]) extends SchemaRegistry {
    override def add(blueprint: Blueprint): Task[Digest] = {
      val digest: Digest = blueprint.digest
      ref.update(_.+(digest -> blueprint)).as(digest)
    }

    override def drop(digest: Digest): UIO[Boolean] =
      ref.modify(map => if (map.contains(digest)) (true, map - digest) else (false, map))

    override def get(id: Digest): Task[Option[Blueprint]] = ref.get.map(_.get(id))

    override def list(index: Int, max: Int): Task[List[Blueprint]] = ref.get.map(_.values.toList)
  }

  final case class FromRedis(redis: Redis) extends SchemaRegistry {
    override def add(blueprint: Blueprint): Task[Digest] = {
      val digest: Digest = blueprint.digest
      for { _ <- redis.set(digest.hex, blueprint) } yield digest
    }

    override def drop(digest: Digest): Task[Boolean] = redis.del(digest.hex).map(_ > 0)

    override def get(id: Digest): Task[Option[Blueprint]] = redis.get(id.hex).returning[Blueprint]

    override def list(index: RuntimeFlags, max: RuntimeFlags): Task[List[Blueprint]] =
      for {
        hexes      <- redis.keys("*").returning[String]
        blueprints <- ZIO.foreach(hexes)(hex => redis.get(hex).returning[Blueprint])
      } yield blueprints.slice(index, index + max).toList.flatMap(_.toList)
  }

  final case class FromMySQL(source: javax.sql.DataSource, ctx: MysqlZioJdbcContext[SnakeCase]) extends SchemaRegistry {
    import FromMySQL._
    import ctx._

    override def add(blueprint: Blueprint): Task[Digest] = {
      val blueprintSpec =
        BlueprintSpec(digestHex = blueprint.digest.hex, digestAlg = blueprint.digest.alg, blueprint = blueprint)

      val sql = quote(query[BlueprintSpec].insertValue(lift(blueprintSpec)))
      ctx.run(sql).provide(ZLayer.succeed(source)).as(blueprint.digest)
    }

    override def drop(digest: Digest): Task[Boolean] = {
      val sql = quote(filterByDigest(digest).update(_.dropped -> lift(Option(new Timestamp(new Date().getTime)))))

      ctx.run(sql).provide(ZLayer.succeed(source)).map(_ > 0)
    }

    override def get(digest: Digest): Task[Option[Blueprint]] = {
      val sql = quote(filterByDigest(digest).map(_.blueprint))

      ctx.run(sql).provide(ZLayer.succeed(source)).map(_.headOption)
    }

    override def list(index: Int, max: Int): Task[List[Blueprint]] = {
      val sql = quote(query[BlueprintSpec].drop(lift(index)).take(lift(max)).map(_.blueprint))

      ctx.run(sql).provide(ZLayer.succeed(source))
    }

    private def filterByDigest(digest: Digest): Quoted[EntityQuery[BlueprintSpec]] =
      quote(query[BlueprintSpec].filter(b => b.digestHex == lift(digest.hex) && b.digestAlg == lift(digest.alg)))
  }

  object FromMySQL {
    implicit val blueprintEncoding: MappedEncoding[Blueprint, Array[Byte]] =
      MappedEncoding[Blueprint, Array[Byte]](_.toJson.getBytes)
    implicit val blueprintDecoder: MappedEncoding[Array[Byte], Blueprint]  =
      MappedEncoding[Array[Byte], Blueprint](bytes => new String(bytes).fromJson[Blueprint].toOption.get)
    implicit val digestEncoding: MappedEncoding[Digest, String]            = MappedEncoding[Digest, String](_.hex)
    implicit val digestDecoder: MappedEncoding[String, Digest] = MappedEncoding[String, Digest](Digest.fromHex)

    implicit val digestAlgEncoder: MappedEncoding[Digest.Algorithm, String] =
      MappedEncoding[Digest.Algorithm, String](_.name)

    implicit val digestAlgDecoder: MappedEncoding[String, Digest.Algorithm] =
      MappedEncoding[String, Digest.Algorithm](Digest.Algorithm.fromString(_).get)

    sealed trait BlueprintFormat

    case class BlueprintSpec(
      id: Option[Int] = None,
      digestHex: String,
      digestAlg: Digest.Algorithm,
      blueprint: Blueprint,
      blueprintFormat: BlueprintFormat = BlueprintFormat.Json,
      created: Option[Timestamp] = None,
      dropped: Option[Timestamp] = None,
    )

    object BlueprintFormat {
      implicit val encoding: MappedEncoding[BlueprintFormat, String] =
        MappedEncoding[BlueprintFormat, String] { case Json => "json" }
      implicit val decoder: MappedEncoding[String, BlueprintFormat]  =
        MappedEncoding[String, BlueprintFormat] { case "json" => BlueprintFormat.Json }

      case object Json extends BlueprintFormat
    }

    object BlueprintSpec {}
  }
}
