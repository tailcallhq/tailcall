package tailcall.server

import tailcall.runtime.ast.Blueprint
import tailcall.server.service.BinaryDigest.Digest
import tailcall.server.service.{BinaryDigest, SchemaRegistry}
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}
import zio.json.EncoderOps

object Main extends ZIOAppDefault {
  // TODO: use API DSL
  val registery = Http.collectZIO[Request] {
    case req @ Method.PUT -> !! / "schema" => for {
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

    case Method.DELETE -> !! / "schemas" / digest => for {
        found <- SchemaRegistry.drop(Digest.fromHex(digest))
        _     <- ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found")).when(found)
      } yield Response.ok

    case Method.GET -> !! / "schemas" / digest => for {
        schema    <- SchemaRegistry.get(Digest.fromHex(digest))
        blueprint <- schema match {
          case Some(blueprint) => ZIO.succeed(blueprint)
          case None            => ZIO.fail(HttpError.NotFound(s"Schema ${digest} not found"))
        }
      } yield Response.json(blueprint.toJson)
  }

  val sanitized = registery.mapError {
    case error: HttpError => Response.fromHttpError(error)
    case error            => Response.fromHttpError(HttpError.InternalServerError(cause = Option(error)))
  }

  override val run = Server.serve(sanitized).exitCode
    .provide(Server.default, SchemaRegistry.memory, BinaryDigest.algorithm("SHA-256"))
}
