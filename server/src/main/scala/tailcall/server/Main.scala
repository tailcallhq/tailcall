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
  val registry = Http.collectZIO[Request] {
    case req @ Method.PUT -> !! / "schemas" => for {
        body      <- req.body.asCharSeq
        blueprint <- Blueprint.decode(body) match {
          case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
          case Right(value) => ZIO.succeed(value)
        }
        digest    <- SchemaRegistry.add(blueprint)
      } yield Response.json(digest.toHex.toJson)

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

    case Method.GET -> !! / "health" => ZIO.succeed(Response.ok)
  }

  private val graphiql = Http.fromResource("graphiql.html")

  val gql = Http.collectRoute[Request] { case Method.GET -> !! / "graphiql" => graphiql }

  def sanitized[R](http: HttpApp[R, Throwable]): App[R] =
    http.mapError {
      case error: HttpError => Response.fromHttpError(error)
      case error            => Response.fromHttpError(HttpError.InternalServerError(cause = Option(error)))
    }

  val adminServer = Server.serve(sanitized(registry)).provide(
    ServerConfig.live.map(_.update(_.port(8080))),
    Server.live,
    BinaryDigest.algorithm("SHA-256"),
    SchemaRegistry.persistent(this.getClass.getResource("/").getPath)
  )

  val userServer: ZIO[Any, Throwable, Nothing] = Server.serve(sanitized(gql))
    .provide(ServerConfig.live.map(_.update(_.port(8081))), Server.live)

  override val run = (adminServer zipPar userServer).exitCode
}
