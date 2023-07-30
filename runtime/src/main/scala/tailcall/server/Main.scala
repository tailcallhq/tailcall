package tailcall.server

import caliban.wrappers.ApolloPersistedQueries
import caliban.{GraphQLResponse, Value}
import tailcall.registry.{InterpreterRegistry, SchemaRegistry}
import tailcall.runtime.service._
import zio._
import zio.http._
import zio.http.model.{HttpError, Method, Status}
import zio.json.EncoderOps

object Main extends ZIOAppDefault {

  override val run = GraphQLConfig.bootstrap { config =>
    // Use in-memory schema registry if no database is configured
    val registryService = config.database
      .fold(SchemaRegistry.memory)(db => SchemaRegistry.mysql(db.host, db.port, db.username, db.password))

    // Use file-based interpreter registry if a file is configured
    val interpreterService = config.file match {
      case Some(path) => InterpreterRegistry.file(path)
      case None       => InterpreterRegistry.live
    }

    Server.install(server(Duration.fromMillis(config.globalResponseTimeout)))
      .flatMap(port => ZIO.log(s"Server started: http://localhost:${port}/graphql") *> ZIO.never).provide(
        ServerConfig.live.update(_.port(config.port)).update(_.objectAggregator(Int.MaxValue)),
        registryService,
        GraphQLGenerator.default,
        ApolloPersistedQueries.live,
        Server.live,
        interpreterService,
        ConfigFileIO.default,
        HttpClient.default(config.allowedHeaders),
      )
  }

  private def jsonError(message: String, status: Status = Status.InternalServerError): Response = {
    val response = GraphQLResponse(data = Value.NullValue, errors = List(Value.StringValue(message)))
    Response.json(response.toJson).setStatus(status)
  }

  private def server(timeout: Duration) =
    (AdminServer.rest ++ Http.collectRoute[Request] {
      case Method.POST -> !! / "graphql"                                          => AdminServer.graphQL
      case Method.POST -> !! / "graphql" / _                                      => GenericServer.graphQL(timeout)
      case req @ Method.GET -> !! / "graphql" / _ if req.url.queryParams.nonEmpty => GenericServer.graphQL(timeout)
      case Method.GET -> _                                                        => Http.fromResource("graphiql.html")
    }).tapErrorZIO(error => ZIO.logErrorCause(s"HttpError", Cause.fail(error))).mapError {
      case error: HttpError => jsonError(error.message, error.status)
      case error            => jsonError(error.getMessage)
    }
}
