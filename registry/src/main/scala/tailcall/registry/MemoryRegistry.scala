package tailcall.registry

import tailcall.runtime.model.{Blueprint, Digest}
import zio.{Ref, Task, UIO, ZIO}

final case class MemoryRegistry(ref: Ref[List[(String, Blueprint)]]) extends SchemaRegistry {
  override def add(blueprint: Blueprint): Task[Digest] = {
    val digest: Digest = blueprint.digest
    ref.get.map(_.find(_._1 == digest.hex)).flatMap {
      case Some(_) => ZIO.succeed(digest)
      case None    => ref.update((digest.hex -> blueprint) :: _).as(digest)

    }
  }

  override def drop(hex: String): UIO[Boolean] = {
    ref.modify { map =>
      val exists  = map.exists(_._1.startsWith(hex))
      val removed = map.filter { case (key, _) => !key.startsWith(hex) }
      exists -> removed
    }
  }

  override def get(hex: String): Task[Option[Blueprint]] = ref.get.map(_.find(_._1.startsWith(hex)).map(_._2))

  override def list(index: Int, max: Int): Task[List[Blueprint]] = ref.get.map(_.map(_._2))
}
