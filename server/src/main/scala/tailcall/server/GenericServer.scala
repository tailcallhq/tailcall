package tailcall.server

import caliban.CalibanError
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service.{DataLoader, GraphQLGenerator}
import tailcall.server.InterpreterDataLoader.{InterpreterLoader, load}
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

  def graphQL: Http[HttpClient with InterpreterLoader with GraphQLGenerator, Throwable, Request, Response] =
    Http.collectZIO[Request] { case req @ Method.POST -> !! / "graphql" / id =>
      for {
        blueprintData <- load(id)
        query         <- GraphQLUtils.decodeQuery(req.body)
        res           <- blueprintData.interpreter.execute(query).provideLayer(DataLoader.http(Option(req)))
          .map(res => res.copy(errors = res.errors.map(toBetterError(_)))).timeoutFail(HttpError.RequestTimeout(
            s"Request timed out after ${blueprintData.timeout}ms"
          ))(blueprintData.timeout.millis)
        _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logWarningCause("GraphQLExecutionError", Cause.fail(error)))
      } yield Response.json(res.toJson)
    }
}
