package tailcall.server

import caliban.{CalibanError, GraphQL}
import caliban.wrappers.ApolloPersistedQueries.{ApolloPersistence, apolloPersistedQueries}
import caliban.wrappers.ApolloTracing.apolloTracing
import caliban.wrappers.Wrappers.printSlowQueries
import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service.{GraphQLGenerator, HttpContext}
import tailcall.server.internal.GraphQLUtils
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}
import zio.json.EncoderOps

object GenericServer {
  private def toBetterError(error: CalibanError): CalibanError = {
    error match {
      case error: CalibanError.ExecutionError  => error.copy(msg = "Orchestration Failure")
      case error: CalibanError.ParsingError    => error
      case error: CalibanError.ValidationError => error
    }
  }
  def graphQL: Http[
    HttpClient with GraphQLGenerator with ApolloPersistence with SchemaRegistry with GraphQLConfig,
    Throwable,
    Request,
    Response,
  ] =
    Http.collectZIO[Request] { case req @ method -> !! / "graphql" / id =>
      for {
        config         <- ZIO.service[GraphQLConfig]
        gReq           <-
          if (req.url.queryParams != QueryParams.empty) GraphQLUtils.decodeRequest(req.url.queryParams)
          else GraphQLUtils.decodeRequest(req.body)
        persistence    <- ZIO.service[ApolloPersistence]
        maybeBlueprint <- if (id == "0") SchemaRegistry.list(0, 1).map(_.headOption) else SchemaRegistry.get(id)
        blueprint      <- ZIO.fromOption(maybeBlueprint)
          .orElseFail(HttpError.BadRequest(s"Blueprint ${id} has not been published yet."))
        gql            <- blueprint.toGraphQL
        gql            <- ZIO.succeed {
          var _gql: GraphQL[HttpContext with ApolloPersistence] = gql
          if (config.enableTracing) _gql = _gql @@ apolloTracing
          config.slowQueryDuration match {
            case Some(duration) => _gql = _gql @@ printSlowQueries(duration)
            case None           => ()
          }
          if (config.persistedQueries) _gql = _gql @@ apolloPersistedQueries
          _gql
        }
        interpreter    <- gql.interpreter
        res            <- (for {
          res <- interpreter.executeRequest(gReq).map(res => res.copy(errors = res.errors.map(toBetterError)))
            .timeoutFail(HttpError.RequestTimeout(s"Request timed out after ${config.globalResponseTimeout}ms"))(
              config.globalResponseTimeout
            )
          _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logWarningCause("GraphQLExecutionError", Cause.fail(error)))
          maxAge <- HttpContext.getState.map(_.cacheMaxAge)
          jsonResponse = Response.json(res.toJson)
        } yield
          if (method == Method.POST || res.errors.nonEmpty) jsonResponse
          else jsonResponse.withCacheControlMaxAge(maxAge.getOrElse(0 seconds)))
          .provideLayer(HttpContext.live(Option(req)) ++ ZLayer.succeed(persistence))
      } yield res
    }
}
