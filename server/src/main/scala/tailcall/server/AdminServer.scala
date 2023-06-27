package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.Blueprint
import tailcall.server.internal.GraphQLUtils
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}
import zio.json.EncoderOps

import java.nio.charset.StandardCharsets

object AdminServer {
  val rest = Http.collectZIO[Request] {
    case req @ Method.PUT -> !! / "schemas" => for {
        body      <- req.body.asString(StandardCharsets.UTF_8)
        blueprint <- Blueprint.decode(body) match {
          case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
          case Right(value) => ZIO.succeed(value)
        }
        digest    <- SchemaRegistry.add(blueprint)
      } yield Response.json(digest.toJson)

    case Method.GET -> !! / "schemas" => for {
        list <- SchemaRegistry.list(0, Int.MaxValue)
      } yield Response.json(list.toJson)

    case req @ Method.DELETE -> !! / "schemas" / hex => for {
        found <- SchemaRegistry.drop(hex)
        _     <- ZIO.fail(HttpError.NotFound(req.url.encode)).when(found)
      } yield Response.ok

    case req @ Method.GET -> !! / "schemas" / hex => for {
        schema    <- SchemaRegistry.get(hex)
        blueprint <- schema match {
          case Some(blueprint) => ZIO.succeed(blueprint)
          case None            => ZIO.fail(HttpError.NotFound(req.url.encode))
        }
      } yield Response.json(blueprint.toJson)

    case Method.GET -> !! / "health" => ZIO.succeed(Response.ok)
  }

  val graphQL = Http.collectZIO[Request] { case req =>
    for {
      gReq        <- GraphQLUtils.decodeRequest(req.body)
      interpreter <- AdminGraphQL.graphQL.interpreter
      res         <- interpreter.executeRequest(gReq)
      _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logWarningCause("GraphQLExecutionError", Cause.fail(error)))
    } yield Response.json(res.toJson)
  }

}
