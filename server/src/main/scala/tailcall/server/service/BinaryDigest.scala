package tailcall.server.service

import zio.json.{DeriveJsonDecoder, DeriveJsonEncoder, JsonDecoder, JsonEncoder}
import zio.schema.Schema
import zio.{ULayer, ZLayer}

import java.security.MessageDigest

trait BinaryDigest {
  def digestWith[A](a: A, encoder: JsonEncoder[A]): BinaryDigest.Digest
  final def digest[A](a: A)(implicit encoder: JsonEncoder[A]): BinaryDigest.Digest = digestWith(a, encoder)
  final def digest[A](a: A)(implicit schema: Schema[A]): BinaryDigest.Digest       =
    digestWith(a, zio.schema.codec.JsonCodec.jsonEncoder(schema))
}

object BinaryDigest {
  final case class Digest(value: Array[Byte]) {
    def toHex: String = value.map("%02x".format(_)).mkString
  }

  object Digest {
    implicit val encoder: JsonEncoder[Digest] = DeriveJsonEncoder.gen[Digest]
    implicit val decoder: JsonDecoder[Digest] = DeriveJsonDecoder.gen[Digest]

    def fromHex(hex: String): Digest = Digest(hex.grouped(2).map(Integer.parseInt(_, 16).toByte).toArray)
  }

  def algorithm(name: String): ULayer[BinaryDigest] =
    ZLayer.succeed(new BinaryDigest {
      override def digestWith[A](a: A, encoder: JsonEncoder[A]): Digest = {
        val encoded = String.valueOf(encoder.encodeJson(a)).getBytes()
        Digest(MessageDigest.getInstance(name).digest(encoded))
      }
    })
}
