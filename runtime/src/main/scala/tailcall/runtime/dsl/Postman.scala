package tailcall.runtime.dsl

import tailcall.runtime.dsl.Postman.Collection
import tailcall.runtime.http.{Method, Scheme}
import zio.json.ast.Json
import zio.json.{DecoderOps, DeriveJsonCodec, JsonCodec, jsonHint}

// scalafmt: { maxColumn = 160 }
final case class Postman(collection: Collection)
object Postman {
  final case class Header(key: String, value: String)

  @jsonHint("body")
  final private case class BodyRaw(raw: String)
  final case class Body(raw: Json)
  final case class Url(protocol: Scheme, host: List[String], path: List[String], query: List[(String, String)] = Nil, variable: List[String] = Nil)
  final case class Request(method: Method, header: List[Header], body: Option[Body], url: Option[Url])
  final case class Response(status: String, header: List[Header], body: String)
  final case class Item(name: String, id: String, request: Request, response: List[Response])
  final case class Info(name: String, schema: String, updatedAt: String)
  final case class Collection(info: Info, item: List[Item])

  implicit private[Postman] lazy val headerCodec: JsonCodec[Header]         = DeriveJsonCodec.gen[Header]
  implicit private[Postman] lazy val responseCodec: JsonCodec[Response]     = DeriveJsonCodec.gen[Response]
  implicit private[Postman] lazy val urlCodec: JsonCodec[Url]               = DeriveJsonCodec.gen[Url]
  implicit private[Postman] lazy val bodyCodec: JsonCodec[Body]             = DeriveJsonCodec.gen[BodyRaw]
    .transformOrFail[Body]({ case BodyRaw(value) => value.fromJson[Json].map(Body(_)) }, { case Body(value) => BodyRaw(value.toString) })
  implicit private[Postman] lazy val requestCodec: JsonCodec[Request]       = DeriveJsonCodec.gen[Request]
  implicit private[Postman] lazy val itemCodec: JsonCodec[Item]             = DeriveJsonCodec.gen[Item]
  implicit private[Postman] lazy val infoCodec: JsonCodec[Info]             = DeriveJsonCodec.gen[Info]
  implicit private[Postman] lazy val collectionCodec: JsonCodec[Collection] = DeriveJsonCodec.gen[Collection]
  implicit lazy val postmanCodec: JsonCodec[Postman]                        = DeriveJsonCodec.gen[Postman]

//  val item = Item(
//    name = "test",
//    id = "test",
//    request = Request(
//      method = Method.GET,
//      header = List(Header("Content-Type", "application/json")),
//      body = Body(Json.Obj("test" -> Json.Num(1))),
//      url = Url(protocol = Scheme.Http, host = List("localhost"), path = List("test"), query = List("test" -> "test"), variable = List()),
//    ),
//    response = List(Response(status = "200", header = List(Header("Content-Type", "application/json")), body = "test")),
//  )
}
