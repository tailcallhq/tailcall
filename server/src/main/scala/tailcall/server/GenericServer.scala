package tailcall.server

import caliban.CalibanError
import tailcall.runtime.service.DataLoader
import tailcall.server.InterpreterDataLoader.load
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

  def graphQL =
    Http.collectZIO[Request] { case req @ Method.POST -> !! / "graphql" / id =>
      for {
        result <- load(id)
        timeout = result._2.flatMap(blueprint => blueprint.server.globalResponseTimeout).getOrElse(10000)
        query <- GraphQLUtils.decodeQuery(req.body)
        res   <- result._1.execute(query).provideLayer(DataLoader.http(Option(req)))
          .map(res => res.copy(errors = res.errors.map(toBetterError(_))))
          .timeoutFail(HttpError.RequestTimeout(s"Request timed out after ${timeout}ms"))(timeout.millis)
        _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logErrorCause("GraphQLExecutionError", Cause.fail(error)))
      } yield Response.json(res.toJson)
    }
}
