package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}

object Main extends ZIOAppDefault {
  val server = (AdminServer.rest ++ Http.collectRoute[Request] {
    case Method.POST -> !! / "graphql"     => AdminServer.graphQL
    case Method.POST -> !! / "graphql" / _ => GenericServer.graphQL
    case Method.GET -> _                   => Http.fromResource("graphiql.html")
  }).tapErrorZIO(error => ZIO.logWarningCause(s"HttpError", Cause.fail(error))).mapError {
    case error: HttpError => Response.fromHttpError(error)
    case error            => Response.fromHttpError(HttpError.InternalServerError(cause = Option(error)))
  }

  override val run = Server.serve(server).exitCode.provide(
    ServerConfig.live.update(_.port(SchemaRegistry.PORT)),
    SchemaRegistry.memory,
    GraphQLGenerator.default,
    HttpClient.default,
    Server.live
  )
}
