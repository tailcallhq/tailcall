package tailcall.server

import caliban.{CalibanError, GraphQLInterpreter}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.{DataLoader, GraphQLGenerator}
import zio._
import zio.http.model.HttpError

object InterpreterDataLoader {
  type InterpreterLoader = DataLoader[
    GraphQLGenerator,
    Throwable,
    String,
    (GraphQLInterpreter[HttpDataLoader, CalibanError], Option[Blueprint]),
  ]

  def load(digestId: String): ZIO[
    InterpreterLoader with GraphQLGenerator,
    Throwable,
    (GraphQLInterpreter[HttpDataLoader, CalibanError], Option[Blueprint]),
  ] = ZIO.serviceWithZIO[InterpreterLoader](_.load(digestId))

  def default: ZLayer[SchemaRegistry, Nothing, InterpreterLoader] =
    ZLayer {
      for {
        schemaRegistry <- ZIO.service[SchemaRegistry]
        gqlCache       <- Ref.make(
          Map.empty[String, Promise[Throwable, (GraphQLInterpreter[HttpDataLoader, CalibanError], Option[Blueprint])]]
        )
        resolver = (digestId: String) =>
          for {
            blueprint   <- schemaRegistry.get(Digest.fromHex(digestId))
            result      <- blueprint match {
              case Some(value) => value.toGraphQL
              case None        => ZIO.fail(HttpError.BadRequest(s"Blueprint ${digestId} has not been published yet."))
            }
            interpreter <- result.interpreter
          } yield (interpreter, blueprint)
      } yield DataLoader(gqlCache, resolver)
    }
}
