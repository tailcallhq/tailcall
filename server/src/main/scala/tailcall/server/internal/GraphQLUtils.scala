package tailcall.server.internal

import caliban.GraphQLRequest
import zio._
import zio.http._
import zio.http.model.HttpError
import zio.json.DecoderOps

import java.nio.charset.StandardCharsets

object GraphQLUtils {
  def decodeQuery(body: Body): ZIO[Any, Throwable, String] =
    for {
      text  <- body.asString(StandardCharsets.UTF_8)
      req   <- text.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
      query <- req.query match {
        case Some(value) => ZIO.succeed(value)
        case None        => ZIO.fail(HttpError.BadRequest("Query is required"))
      }
    } yield query
}
