package tailcall.server

import caliban.wrappers.ApolloPersistedQueries
import caliban.wrappers.ApolloPersistedQueries.ApolloPersistence
import caliban.{GraphQLResponse, Value}
import tailcall.registry.{InterpreterRegistry, SchemaRegistry}
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio._
import zio.http._
import zio.http.model.{HttpError, Method, Status}
import zio.json.EncoderOps

object Main extends ZIOAppDefault {
  private type ServerEnv = SchemaRegistry
    with GraphQLGenerator with HttpClient with ApolloPersistence with InterpreterRegistry

  override val run = GraphQLConfig.bootstrap { config =>
    // Use in-memory schema registry if no database is configured
    val registry = config.database
      .fold(SchemaRegistry.memory)(db => SchemaRegistry.mysql(db.host, db.port, db.username, db.password))

    val publicServer: ZIO[ServerEnv, Throwable, Nothing] = Server
      .install(publicGraphQLHttp(Duration.fromMillis(config.globalResponseTimeout)))
      .flatMap(port => ZIO.log(s"GraphQL server started: http://localhost:${port}/graphql") *> ZIO.never)
      .provideSome(Server.live, ServerConfig.live.update(_.port(config.port)))

    val privateServer: ZIO[Any, Throwable, Nothing] = Server.install(toApp(AdminServer.rest))
      .flatMap(port => ZIO.log(s"Admin server started on port: ${port}") *> ZIO.never)
      .provide(Server.live, ServerConfig.live.update(_.port(config.adminPort)), registry)

    privateServer.zipPar(publicServer).provide(
      registry,
      GraphQLGenerator.default,
      HttpClient.cachedDefault(config.httpCacheSize, config.allowedHeaders),
      ApolloPersistedQueries.live,
      InterpreterRegistry.live,
    )
  }

  private def jsonError(message: String, status: Status = Status.InternalServerError): Response = {
    val response = GraphQLResponse(data = Value.NullValue, errors = List(Value.StringValue(message)))
    Response.json(response.toJson).setStatus(status)
  }

  private def publicGraphQLHttp(timeout: Duration) =
    toApp {
      Http.collectRoute[Request] {
        case Method.POST -> !! / "graphql"                                          => AdminServer.graphQL
        case Method.POST -> !! / "graphql" / _                                      => GenericServer.graphQL(timeout)
        case req @ Method.GET -> !! / "graphql" / _ if req.url.queryParams.nonEmpty => GenericServer.graphQL(timeout)
        case Method.GET -> _ => Http.fromResource("graphiql.html")
      }
    }

  private def toApp[R](app: HttpApp[R, Throwable]): Http[R, Response, Request, Response] =
    app.tapErrorZIO(error => ZIO.logErrorCause(s"HttpError", Cause.fail(error))).mapError {
      case error: HttpError => jsonError(error.message, error.status)
      case error            => jsonError(error.getMessage)
    }
}
