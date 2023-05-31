package tailcall.server

import caliban.{GraphQLResponse, Value}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio._
import zio.http._
import zio.http.model.{HttpError, Method, Status}
import zio.json.EncoderOps

object Main extends ZIOAppDefault {
  override val run   = GraphQLConfig.bootstrap { config =>
    Server.install(server).flatMap(port => ZIO.log(s"Server started: http://localhost:${port}/graphql") *> ZIO.never)
      .exitCode.provide(
        ServerConfig.live.update(_.port(config.port)).update(_.objectAggregator(Int.MaxValue)),

        // Use in-memory schema registry if no database is configured
        config.database
          .fold(SchemaRegistry.memory)(db => SchemaRegistry.mysql(db.host, db.port, db.username, db.password)),
        GraphQLGenerator.default,
        HttpClient.cachedDefault(config.httpCacheSize),
        Server.live,
        BlueprintDataLoader.live(config),
      )
  }
  private val server = (AdminServer.rest ++ Http.collectRoute[Request] {
    case Method.POST -> !! / "graphql"     => AdminServer.graphQL
    case Method.POST -> !! / "graphql" / _ => GenericServer.graphQL
    case Method.GET -> _                   => Http.fromResource("graphiql.html")
  }).tapErrorZIO(error => ZIO.logErrorCause(s"HttpError", Cause.fail(error))).mapError {
    case error: HttpError => jsonError(error.message)
    case error            => jsonError(error.getMessage)
  }

  private def jsonError(message: String, status: Status = Status.InternalServerError): Response = {
    val response = GraphQLResponse(data = Value.NullValue, errors = List(Value.StringValue(message)))
    Response.json(response.toJson).setStatus(status)
  }
}
