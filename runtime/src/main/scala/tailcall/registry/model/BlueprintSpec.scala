package tailcall.registry.model

import io.getquill.MappedEncoding
import tailcall.registry.model.BlueprintSpec.Format
import tailcall.runtime.model.{Blueprint, Digest}
import zio.json.{DecoderOps, EncoderOps}

import java.sql.Timestamp

case class BlueprintSpec(
  id: Option[Int] = None,
  digestHex: String,
  digestAlg: Digest.Algorithm,
  blueprint: Blueprint,
  blueprintFormat: Format = Format.Json,
  created: Option[Timestamp],
  dropped: Option[Timestamp] = None,
)

object BlueprintSpec {
  implicit val blueprintEncoder: MappedEncoding[Blueprint, Array[Byte]] =
    MappedEncoding[Blueprint, Array[Byte]](_.toJson.getBytes)

  implicit val blueprintDecoder: MappedEncoding[Array[Byte], Blueprint] =
    MappedEncoding[Array[Byte], Blueprint](bytes => new String(bytes).fromJson[Blueprint].toOption.get)

  implicit val digestAlgEncoder: MappedEncoding[Digest.Algorithm, String] =
    MappedEncoding[Digest.Algorithm, String](_.name)

  implicit val digestAlgDecoder: MappedEncoding[String, Digest.Algorithm] =
    MappedEncoding[String, Digest.Algorithm](Digest.Algorithm.fromString(_).get)

  sealed trait Format {
    def name: String
  }

  object Format {
    implicit val encoding: MappedEncoding[Format, String] = MappedEncoding[Format, String](_.name)
    implicit val decoder: MappedEncoding[String, Format]  = MappedEncoding[String, Format](fromName)

    def fromName(name: String): Format =
      name match {
        case Json.name => Json
        case _         => Json
      }

    def json: Format = Json

    case object Json extends Format {
      override val name: String = "json"
    }
  }
}
