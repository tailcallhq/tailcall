package tailcall.server.service

import zio.json.{DeriveJsonDecoder, DeriveJsonEncoder, JsonDecoder, JsonEncoder}
import zio.{ULayer, ZIO, ZLayer}

import java.security.MessageDigest

trait BinaryDigest {
  def digestWith[A](a: A, encoder: JsonEncoder[A]): BinaryDigest.Digest
  final def digest[A](a: A)(implicit encoder: JsonEncoder[A]): BinaryDigest.Digest = digestWith(a, encoder)
}

object BinaryDigest {
  final case class Digest(hex: String) {
    def getBytes: Array[Byte] = hex.getBytes
  }

  object Digest {
    implicit val encoder: JsonEncoder[Digest] = DeriveJsonEncoder.gen[Digest]
    implicit val decoder: JsonDecoder[Digest] = DeriveJsonDecoder.gen[Digest]

    def fromHex(hex: String): Digest = Digest(hex)
  }

  def sha256: ULayer[BinaryDigest] = algorithm("SHA-256")

  def algorithm(name: String): ULayer[BinaryDigest] =
    ZLayer.succeed(new BinaryDigest {
      override def digestWith[A](a: A, encoder: JsonEncoder[A]): Digest = {
        val encoded = String.valueOf(encoder.encodeJson(a)).getBytes()
        Digest(MessageDigest.getInstance(name).digest(encoded).map("%02x".format(_)).mkString)
      }
    })

  def digest[A](a: A)(implicit encoder: JsonEncoder[A]): ZIO[BinaryDigest, Nothing, Digest] =
    ZIO.serviceWith[BinaryDigest](_.digest(a))
}
