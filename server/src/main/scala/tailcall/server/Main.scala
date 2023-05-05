package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio._
import zio.http._
import zio.http.model.{HttpError, Method, Status}

object Main extends ZIOAppDefault {
  val server = (AdminServer.rest ++ Http.collectRoute[Request] {
    case Method.POST -> !! / "graphql"     => AdminServer.graphQL
    case Method.POST -> !! / "graphql" / _ => GenericServer.graphQL
    case Method.GET -> _                   => Http.fromResource("graphiql.html")
  }).tapErrorZIO(error => ZIO.logErrorCause(s"HttpError", Cause.fail(error))).mapError {
    case error: HttpError => Response(status = error.status, body = Body.fromString(error.message))
    case error            => Response(status = Status.InternalServerError, body = Body.fromString(error.getMessage))
  }

  override val run = Server.install(server)
    .flatMap(port => ZIO.log(s"Server started: http://localhost:${port}/graphql") *> ZIO.never).exitCode.provide(
      ServerConfig.live.update(_.port(SchemaRegistry.PORT)).update(_.objectAggregator(Int.MaxValue)),
      SchemaRegistry.memory,
      GraphQLGenerator.default,
      HttpClient.cachedDefault(1000),
      Server.live,
      BlueprintDataLoader.live(enableTracing = true, Some(1000)),
    )
}
