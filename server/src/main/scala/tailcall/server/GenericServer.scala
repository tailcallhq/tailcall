package tailcall.server

import caliban.CalibanError
import caliban.wrappers.ApolloPersistedQueries.ApolloPersistence
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service.{GraphQLGenerator, HttpContext}
import tailcall.server.BlueprintDataLoader.{InterpreterLoader, load}
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
    HttpClient with GraphQLGenerator with InterpreterLoader with ApolloPersistence,
    Throwable,
    Request,
    Response,
  ] =
    Http.collectZIO[Request] { case req @ method -> !! / "graphql" / id =>
      for {
        blueprintData <- load(id)
        gReq          <-
          if (req.url.queryParams != QueryParams.empty) GraphQLUtils.decodeRequest(req.url.queryParams)
          else GraphQLUtils.decodeRequest(req.body)
        persistence   <- ZIO.service[ApolloPersistence]
        res           <- (for {
          res <- blueprintData.interpreter.executeRequest(gReq)
            .map(res => res.copy(errors = res.errors.map(toBetterError))).timeoutFail(HttpError.RequestTimeout(
              s"Request timed out after ${blueprintData.timeout}ms"
            ))(blueprintData.timeout.millis)
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
