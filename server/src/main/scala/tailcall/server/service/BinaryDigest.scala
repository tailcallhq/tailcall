package tailcall.server.service

import zio.json.{DeriveJsonDecoder, DeriveJsonEncoder, JsonDecoder, JsonEncoder}
import zio.{ULayer, ZIO, ZLayer}

import java.security.MessageDigest

trait BinaryDigest {
  def digestWith[A](a: A, encoder: JsonEncoder[A]): BinaryDigest.Digest
  final def digest[A](a: A)(implicit encoder: JsonEncoder[A]): BinaryDigest.Digest = digestWith(a, encoder)
}

object BinaryDigest {
  sealed trait Algorithm {
    self =>
    final def name: String = self match { case Algorithm.SHA_256 => "SHA-256" }
  }

  object Algorithm {
    case object SHA_256 extends Algorithm

    implicit val encoder: JsonEncoder[Algorithm] = DeriveJsonEncoder.gen[Algorithm]
    implicit val decoder: JsonDecoder[Algorithm] = DeriveJsonDecoder.gen[Algorithm]

    def fromString(s: String): Option[Algorithm] =
      s.toUpperCase match {
        case "SHA-256" => Some(SHA_256)
        case _         => None
      }
  }

  final case class Digest(alg: Algorithm, hex: String) {
    def getBytes: Array[Byte] = hex.getBytes
  }

  object Digest {
    implicit val encoder: JsonEncoder[Digest] = DeriveJsonEncoder.gen[Digest]
    implicit val decoder: JsonDecoder[Digest] = DeriveJsonDecoder.gen[Digest]

    def fromHex(algorithm: Algorithm, hex: String): Digest = Digest(algorithm, hex)
  }

  def sha256: ULayer[BinaryDigest] = algorithm(Algorithm.SHA_256)

  def algorithm(algorithm: Algorithm): ULayer[BinaryDigest] =
    ZLayer.succeed(new BinaryDigest {
      override def digestWith[A](a: A, encoder: JsonEncoder[A]): Digest = {
        val encoded = String.valueOf(encoder.encodeJson(a)).getBytes()
        Digest(algorithm, MessageDigest.getInstance(algorithm.name).digest(encoded).map("%02x".format(_)).mkString)
      }
    })

  def digest[A](a: A)(implicit encoder: JsonEncoder[A]): ZIO[BinaryDigest, Nothing, Digest] =
    ZIO.serviceWith[BinaryDigest](_.digest(a))
}
