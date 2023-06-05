package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.server.internal.GraphQLUtils
import zio._
import zio.http.{HttpError, Method, _}
import zio.json.EncoderOps

import java.nio.charset.StandardCharsets

object AdminServer {
  val rest = Http.collectZIO[Request] {
    case req @ Method.PUT -> Root / "schemas" => for {
        body      <- req.body.asString(StandardCharsets.UTF_8)
        blueprint <- Blueprint.decode(body) match {
          case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
          case Right(value) => ZIO.succeed(value)
        }
        digest    <- SchemaRegistry.add(blueprint)
      } yield Response.json(digest.toJson)

    case Method.GET -> Root / "schemas" => for {
        list <- SchemaRegistry.list(0, Int.MaxValue)
      } yield Response.json(list.toJson)

    case Method.DELETE -> Root / "schemas" / digest => for {
        found <- SchemaRegistry.drop(Digest.fromHex(digest))
        _     <- ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found")).when(found)
      } yield Response.ok

    case Method.GET -> Root / "schemas" / digest => for {
        schema    <- SchemaRegistry.get(Digest.fromHex(digest))
        blueprint <- schema match {
          case Some(blueprint) => ZIO.succeed(blueprint)
          case None            => ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found"))
        }
      } yield Response.json(blueprint.toJson)

    case Method.GET -> Root / "health" => ZIO.succeed(Response.ok)
  }

  val graphQL = Http.collectZIO[Request] { case req =>
    for {
      query       <- GraphQLUtils.decodeQuery(req.body)
      interpreter <- AdminGraphQL.graphQL.interpreter
      res         <- interpreter.execute(query)
      _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logWarningCause("GraphQLExecutionError", Cause.fail(error)))
    } yield Response.json(res.toJson)
  }

}
