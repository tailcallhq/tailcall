package tailcall.registry

import tailcall.runtime.model.{Blueprint, Digest}
import zio.{Ref, Task, UIO}

final case class MemoryRegistry(ref: Ref[Map[Digest, Blueprint]]) extends SchemaRegistry {
  override def add(blueprint: Blueprint): Task[Digest] = {
    val digest: Digest = blueprint.digest
    ref.update(_.+(digest -> blueprint)).as(digest)
  }

  override def drop(digest: Digest): UIO[Boolean] = {
    println(digest)
    ref.modify(map =>
      if (map.exists(_._1.hex.startsWith(digest.prefix))) (true, map.filterNot(_._1.prefix == digest.prefix))
      else (false, map)
    )
  }

  override def get(id: Digest): Task[Option[Blueprint]] =
    ref.get.map(_.filter(_._1.hex.startsWith(id.prefix)).values.toList.headOption)

  override def list(index: Int, max: Int): Task[List[Blueprint]] = ref.get.map(_.values.toList)
}
