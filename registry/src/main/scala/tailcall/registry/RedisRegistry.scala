package tailcall.registry

import tailcall.runtime.model.{Blueprint, Digest}
import zio.redis.Redis
import zio.{Task, ZIO}

final case class RedisRegistry(redis: Redis) extends SchemaRegistry {
  override def add(blueprint: Blueprint): Task[Digest] = {
    val digest: Digest = blueprint.digest
    for { _ <- redis.set(digest.hex, blueprint) } yield digest
  }

  override def drop(digest: Digest): Task[Boolean] = redis.del(digest.hex).map(_ > 0)

  override def get(id: Digest): Task[Option[Blueprint]] = redis.get(id.hex).returning[Blueprint]

  override def list(index: Int, max: Int): Task[List[Blueprint]] =
    for {
      hexes      <- redis.keys("*").returning[String]
      blueprints <- ZIO.foreach(hexes)(hex => redis.get(hex).returning[Blueprint])
    } yield blueprints.slice(index, index + max).toList.flatMap(_.toList)
}
