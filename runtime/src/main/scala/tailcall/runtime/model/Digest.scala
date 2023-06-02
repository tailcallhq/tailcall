package tailcall.runtime.model

import tailcall.runtime.model.Digest.Algorithm
import zio.json.{DeriveJsonDecoder, DeriveJsonEncoder, JsonDecoder, JsonEncoder}
import zio.schema.{DeriveSchema, Schema}

import java.security.MessageDigest

final case class Digest(alg: Algorithm, hex: String) {
  def getBytes: Array[Byte] = hex.getBytes
}

object Digest {
  implicit val encoder: JsonEncoder[Digest] = DeriveJsonEncoder.gen[Digest]
  implicit val decoder: JsonDecoder[Digest] = DeriveJsonDecoder.gen[Digest]
  implicit val schema: Schema[Digest]       = DeriveSchema.gen[Digest]

  def fromBlueprint(blueprint: Blueprint, algorithm: Algorithm = Algorithm.SHA_256): Digest = {
    val encoded = String.valueOf(Blueprint.encode(blueprint)).getBytes()
    Digest(algorithm, MessageDigest.getInstance(algorithm.name).digest(encoded).map("%02x".format(_)).mkString)
  }

  def fromHex(hex: String): Digest = Digest(Algorithm.SHA_256, hex)

  sealed trait Algorithm {
    self =>
    def name: String
  }

  object Algorithm {
    implicit val encoder: JsonEncoder[Algorithm] = DeriveJsonEncoder.gen[Algorithm]
    implicit val decoder: JsonDecoder[Algorithm] = DeriveJsonDecoder.gen[Algorithm]

    def fromString(s: String): Option[Algorithm] =
      s.toUpperCase match {
        case SHA_256.name => Some(SHA_256)
        case _            => None
      }

    case object SHA_256 extends Algorithm {
      final override val name: String = "SHA-256"
    }
  }
}
