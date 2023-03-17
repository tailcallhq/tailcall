package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.ast.Digest.Algorithm
import tailcall.runtime.ast.{Blueprint, Digest}
import tailcall.server.internal.GraphQLUtils
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}
import zio.json.EncoderOps

object AdminServer {
  val rest = Http.collectZIO[Request] {
    case req @ Method.PUT -> !! / "schemas" => for {
        body      <- req.body.asCharSeq
        blueprint <- Blueprint.decode(body) match {
          case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
          case Right(value) => ZIO.succeed(value)
        }
        digest    <- SchemaRegistry.add(blueprint)
      } yield Response.json(digest.toJson)

    case Method.GET -> !! / "schemas" => for {
        list <- SchemaRegistry.list(0, Int.MaxValue)
      } yield Response.json(list.toJson)

    case Method.DELETE -> !! / "schemas" / alg / digest => for {
        algorithm <- ZIO.fromOption(Algorithm.fromString(alg))
          .orElseFail(HttpError.BadRequest(s"Invalid algorithm ${alg}"))
        found     <- SchemaRegistry.drop(Digest.fromHex(algorithm, digest))
        _         <- ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found")).when(found)
      } yield Response.ok

    case Method.GET -> !! / "schemas" / alg / digest => for {
        algorithm <- ZIO.fromOption(Algorithm.fromString(alg))
          .orElseFail(HttpError.BadRequest(s"Invalid algorithm ${alg}"))
        schema    <- SchemaRegistry.get(Digest.fromHex(algorithm, digest))
        blueprint <- schema match {
          case Some(blueprint) => ZIO.succeed(blueprint)
          case None            => ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found"))
        }
      } yield Response.json(blueprint.toJson)

    case Method.GET -> !! / "health" => ZIO.succeed(Response.ok)
  }

  val graphQL = Http.collectZIO[Request] { case req =>
    for {
      query       <- GraphQLUtils.decodeQuery(req.body)
      interpreter <- AdminGraphQL.graphQL.interpreter
      res         <- interpreter.execute(query)
    } yield Response.json(res.toJson)
  }

}
