package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.Digest
import tailcall.runtime.service.DataLoader
import tailcall.server.internal.GraphQLUtils
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}
import zio.json.EncoderOps

object GenericServer {
  def graphQL =
    Http.collectZIO[Request] { case req @ Method.POST -> !! / "graphql" / id =>
      for {
        schema <- if (id == "0") SchemaRegistry.list(0, 1).map(_.headOption) else SchemaRegistry.get(Digest.fromHex(id))
        timeout     <- ZIO.succeed(
          (for {
            blueprint <- schema
            timeout   <- blueprint.server.globalResponseTimeout
          } yield timeout).getOrElse(10000)
        )
        result      <- schema match {
          case Some(value) => value.toGraphQL
          case None        => ZIO.fail(HttpError.BadRequest(s"Blueprint ${id} has not been published yet."))
        }
        query       <- GraphQLUtils.decodeQuery(req.body)
        interpreter <- result.interpreter
        res         <- interpreter.execute(query).provideLayer(DataLoader.http(Option(req)))
          .timeoutFail(HttpError.RequestTimeout(s"Request timed out after ${timeout}ms"))(timeout.millis)
        _ <- ZIO.foreachDiscard(res.errors)(error => ZIO.logWarningCause("GraphQLExecutionError", Cause.fail(error)))
      } yield Response.json(res.toJson)
    }
}
