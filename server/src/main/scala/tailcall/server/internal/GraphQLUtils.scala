package tailcall.server.internal

import caliban.GraphQLRequest
import zio._
import zio.http._
import zio.http.model.HttpError
import zio.json._
import zio.json.ast.Json

import java.nio.charset.StandardCharsets

object GraphQLUtils {
  def decodeQuery(body: Body): ZIO[Any, Throwable, GraphQLRequest] =
    for {
      text <- body.asString(StandardCharsets.UTF_8)
      req  <- text.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req

  def decodeRequest(queryParams: QueryParams): ZIO[Any, Throwable, GraphQLRequest] = {
    def updateJson(either: Either[String, Json]): Json = {
      either match {
        case Left(_)     => Json.Null
        case Right(json) => json match {
            case obj: Json.Obj =>
              val updatedFields = obj.fields.map { case (key, value) => key -> updateJson(Right(value)) }
              Json.Obj(updatedFields: _*)
            case str: Json.Str => str.asString.fold(json)(parsedString => parsedString.fromJson[Json].getOrElse(json))
            case _             => json
          }
      }
    }

    val updatedJson = updateJson(queryParams.map(x => (x._1, x._2.mkString(","))).toMap.toJsonAST).toJson

    for {
      req <- updatedJson.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req
  }

}
