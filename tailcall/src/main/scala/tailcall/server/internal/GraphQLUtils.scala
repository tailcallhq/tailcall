package tailcall.server.internal

import caliban.{GraphQLRequest, InputValue}
import zio._
import zio.http._
import zio.http.model.HttpError
import zio.json._

import java.nio.charset.StandardCharsets

object GraphQLUtils {
  def decodeRequest(body: Body): ZIO[Any, Throwable, GraphQLRequest] =
    for {
      text <- body.asString(StandardCharsets.UTF_8)
      req  <- text.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req

  def decodeRequest(queryParams: QueryParams): ZIO[Any, Throwable, GraphQLRequest] = {
    val query         = queryParams.get("query").flatMap(_.headOption)
    val extensions    = queryParams.get("extensions").flatMap(_.headOption)
      .flatMap(_.fromJson[Map[String, InputValue]].toOption)
    val variables     = queryParams.get("variables").flatMap(_.headOption)
      .flatMap(_.fromJson[Map[String, InputValue]].toOption)
    val operationName = queryParams.get("operationName").flatMap(_.headOption)
    ZIO.succeed(GraphQLRequest(query, operationName, variables, extensions))
  }

}
