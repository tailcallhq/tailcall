package tailcall.server

import tailcall.runtime.ast.Blueprint
import zio._
import zio.http._
import zio.http.model.{HttpError, Method}

object Main extends ZIOAppDefault {

  // TODO: add all registry routes
  val app = Http.collectZIO[Request] { case req @ Method.PUT -> !! / "registry" / "create" =>
    for {
      body      <- req.body.asCharSeq
      blueprint <- Blueprint.decode(body) match {
        case Left(value)  => ZIO.fail(HttpError.BadRequest(value))
        case Right(value) => ZIO.succeed(value)
      }
    } yield Response.ok
  }

  val sanitized = app.mapError {
    case error: HttpError => Response.fromHttpError(error)
    case error            => Response.fromHttpError(HttpError.InternalServerError(cause = Option(error)))
  }

  override val run = Server.serve(sanitized).provide(Server.default)
}
