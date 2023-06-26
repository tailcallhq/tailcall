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
      text  <- body.asString(StandardCharsets.UTF_8)
      req   <- text.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req

  def decodeRequest(queryParams: QueryParams): ZIO[Any, Throwable, GraphQLRequest] = {
    println(queryParams.map(x => (x._1, x._2.mkString(","))))

    val query = queryParams.map(x => (x._1, x._2.mkString(","))).toMap.toJson
    // Parse the JSON string
    val json  = query.fromJson[Json].getOrElse(Json.Null)

    // Update the JSON object by iterating over its fields
    val updatedJson = json match {
      case obj: Json.Obj =>
        val updatedFields = obj.fields.map { case (key, value) =>
          val updatedValue = value match {
            case str: Json.Str => str.asString.fold(value)(parsedString => parsedString.fromJson[Json].getOrElse(value))
            case _             => value
          }
          key -> updatedValue
        }
        Json.Obj(updatedFields: _*)
      case _             => json
    }

    // Convert the updated JSON object back to a string
    val convertedString = updatedJson.toJson

    val _ = pprint.pprintln(convertedString)
    for {
      req   <- convertedString.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req
  }

}
