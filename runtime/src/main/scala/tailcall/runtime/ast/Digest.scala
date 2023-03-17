package tailcall.runtime.ast

import tailcall.runtime.ast.Digest.Algorithm
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

  def fromHex(hex: String): Digest = Digest(Algorithm.SHA_256, hex)

  def fromBlueprint(blueprint: Blueprint, algorithm: Algorithm = Algorithm.SHA_256): Digest = {
    val encoded = String.valueOf(Blueprint.encode(blueprint)).getBytes()
    Digest(algorithm, MessageDigest.getInstance(algorithm.name).digest(encoded).map("%02x".format(_)).mkString)
  }

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
}
