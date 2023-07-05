package tailcall.runtime.model

import tailcall.runtime.http.{Method, Scheme}
import tailcall.runtime.model.Postman.Collection
import zio.json.ast.Json
import zio.json.{
  DecoderOps,
  DeriveJsonCodec,
  DeriveJsonDecoder,
  DeriveJsonEncoder,
  EncoderOps,
  JsonCodec,
  JsonDecoder,
  JsonEncoder,
  jsonHint,
}

// scalafmt: { maxColumn = 160 }
final case class Postman(collection: Collection)
object Postman {
  final case class Header(key: String, value: String)

  @jsonHint("body")
  final private case class BodyRaw(raw: String)
  final case class Body(raw: Json)
  final case class KeyValue(key: String, value: String, description: Option[String])
  final case class Url(protocol: Option[Scheme], host: List[String], path: List[String], query: List[KeyValue] = Nil, variable: List[KeyValue] = Nil)
  final case class Request(method: Method, header: List[Header], body: Option[Body], url: Option[Url])
  final case class Response(status: String, header: List[Header], body: String)
  sealed trait Item
  object Item {
    final case class FolderItem(name: String, item: List[Item])                                              extends Item
    final case class ValueItem(name: String, id: Option[String], request: Request, response: List[Response]) extends Item
    object FolderItem {
      implicit val encoder: JsonEncoder[FolderItem] = DeriveJsonEncoder.gen[FolderItem]
      implicit val decoder: JsonDecoder[FolderItem] = DeriveJsonDecoder.gen[FolderItem]
    }
    object ValueItem  {
      implicit val encoder: JsonEncoder[ValueItem] = DeriveJsonEncoder.gen[ValueItem]
      implicit val decoder: JsonDecoder[ValueItem] = DeriveJsonDecoder.gen[ValueItem]
    }

    def encode(item: Item): Json = {
      item match {
        case FolderItem(name, item)       => Json.Obj("name" -> Json.Str(name), "item" -> Json.Arr(item.map(encode): _*))
        case item @ ValueItem(_, _, _, _) => item.toJsonAST.getOrElse(Json.Null)
      }
    }

    def decode(json: Json): Either[String, Item] = {
      val isFolder = json match {
        case Json.Obj(fields) => fields.exists { case (name, _) => name == "item" }
        case _                => false
      }
      if (isFolder) json.toJson.fromJson[FolderItem] else json.toJson.fromJson[ValueItem]
    }

    implicit lazy val encoder: JsonEncoder[Item] = JsonEncoder[Json].contramap[Item](encode(_))
    implicit lazy val decoder: JsonDecoder[Item] = JsonDecoder[Json].mapOrFail(decode(_))

  }
  final case class Info(name: String, schema: String, updatedAt: Option[String])
  final case class Collection(info: Info, item: List[Item])

  implicit private[Postman] lazy val headerCodec: JsonCodec[Header]     = DeriveJsonCodec.gen[Header]
  implicit private[Postman] lazy val responseCodec: JsonCodec[Response] = DeriveJsonCodec.gen[Response]
  implicit private[Postman] lazy val urlCodec: JsonCodec[Url]           = DeriveJsonCodec.gen[Url]
  implicit private[Postman] lazy val bodyCodec: JsonCodec[Body]         = DeriveJsonCodec.gen[BodyRaw]
    .transformOrFail[Body]({ case BodyRaw(value) => value.fromJson[Json].map(Body(_)) }, { case Body(value) => BodyRaw(value.toString) })
  implicit private[Postman] lazy val requestCodec: JsonCodec[Request]   = DeriveJsonCodec.gen[Request]
  implicit private[Postman] lazy val variableCodec: JsonCodec[KeyValue] = DeriveJsonCodec.gen[KeyValue]
  implicit private[Postman] lazy val infoCodec: JsonCodec[Info]         = DeriveJsonCodec.gen[Info]
  implicit lazy val collectionCodec: JsonCodec[Collection]              = DeriveJsonCodec.gen[Collection]
  implicit lazy val postmanCodec: JsonCodec[Postman]                    = DeriveJsonCodec.gen[Postman]
}
