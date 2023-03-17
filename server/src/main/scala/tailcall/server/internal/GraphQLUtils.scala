package tailcall.server.internal

import caliban.GraphQLRequest
import zio._
import zio.http._
import zio.http.model.HttpError
import zio.json.DecoderOps

object GraphQLUtils {
  def decode(body: Body): ZIO[Any, Throwable, GraphQLRequest] =
    for {
      text <- body.asCharSeq
      req  <- text.fromJson[GraphQLRequest] match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield req
  def decodeQuery(body: Body): ZIO[Any, Throwable, String]    =
    for {
      text  <- body.asCharSeq
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
