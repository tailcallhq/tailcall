package tailcall.server

import tailcall.registry.SchemaRegistry
import tailcall.runtime.ast.Digest
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
        schema      <- SchemaRegistry.get(Digest.fromHex(id))
        result      <- schema match {
          case Some(value) => value.toGraphQL
          case None        => ZIO.fail(HttpError.NotFound(s"Schema ${id} not found"))
        }
        query       <- GraphQLUtils.decodeQuery(req.body)
        interpreter <- result.interpreter
        res         <- interpreter.execute(query).provideLayer(DataLoader.http)
      } yield Response.json(res.toJson)
    }
}
