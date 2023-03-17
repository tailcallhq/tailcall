package tailcall.cli.service

import tailcall.cli.service.ConfigStore.Key
import zio.rocksdb.RocksDB
import zio.schema.codec.JsonCodec
import zio.schema.{DeriveSchema, Schema}
import zio.{Task, ZIO, ZLayer}

trait ConfigStore {
  def set[A](key: Key[A], value: A): Task[Unit]
  def get[A](key: Key[A]): Task[Option[A]]
}

object ConfigStore {
  sealed trait Key[A] {
    self =>
    final def getBytes: Array[Byte] = JsonCodec.jsonEncoder(Key.schema[A]).encodeJson(self).toString.getBytes
    def name: String
    def schemaA: Schema[A]
  }

  object Key {
    case object RemoteServer extends Key[String] {
      override def schemaA: Schema[String] = Schema[String]
      override def name: String            = "RemoteServer"
    }

    case object RemotePort extends Key[Int] {
      override def schemaA: Schema[Int] = Schema[Int]
      override def name: String         = "RemotePort"
    }

    def schema[A]: Schema[Key[A]] = DeriveSchema.gen[Key[A]]
  }

  final case class Live(rocksDB: RocksDB) extends ConfigStore {
    override def set[A](key: Key[A], value: A): Task[Unit] = {
      val encoderA = JsonCodec.jsonEncoder(key.schemaA)
      rocksDB.put(key.getBytes, encoderA.encodeJson(value).toString.getBytes)
    }

    override def get[A](key: Key[A]): Task[Option[A]] = {
      val decoderA = JsonCodec.jsonDecoder(key.schemaA)
      rocksDB.get(key.getBytes).flatMap {
        case Some(value) => decoderA.decodeJson(new String(value)) match {
            case Left(value)  => ZIO.fail(new RuntimeException(value))
            case Right(value) => ZIO.succeed(Option(value))
          }
        case None        => ZIO.succeed(None)
      }
    }
  }

  def live: ZLayer[RocksDB, Nothing, ConfigStore] = ZLayer.fromFunction(Live(_))
}
