package tailcall.server

import caliban.wrappers.ApolloTracing.apolloTracing
import caliban.wrappers.Wrappers.printSlowQueries
import caliban.{CalibanError, GraphQLInterpreter}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.runtime.service.{DataLoader, GraphQLGenerator, HttpContext}
import zio._
import zio.http.model.HttpError

object BlueprintDataLoader {
  final case class BlueprintData(
    blueprint: Blueprint,
    timeout: Int,
    interpreter: GraphQLInterpreter[HttpContext, CalibanError],
  )

  type InterpreterLoader = DataLoader[GraphQLGenerator, Throwable, String, BlueprintData]

  def load(digestId: String): ZIO[GraphQLGenerator with InterpreterLoader, Throwable, BlueprintData] =
    ZIO.serviceWithZIO[InterpreterLoader](_.load(digestId))

  def default: ZLayer[SchemaRegistry, Nothing, InterpreterLoader] =
    live(enableTracing = false, slowQueriesDuration = None)

  def live(
    enableTracing: Boolean,
    slowQueriesDuration: Option[Int],
  ): ZLayer[SchemaRegistry, Nothing, InterpreterLoader] =
    ZLayer {
      for {
        registry <- ZIO.service[SchemaRegistry]
        dl       <- DataLoader.make[String] { digestId =>
          for {
            maybeBlueprint <- registry.get(Digest.fromHex(digestId))
            blueprint      <- ZIO.fromOption(maybeBlueprint)
              .orElse(ZIO.fail(HttpError.BadRequest(s"Blueprint ${digestId} has not been published yet.")))
            gql            <- blueprint.toGraphQL
            gqlWithTracing     = if (enableTracing) gql @@ apolloTracing else gql
            gqlWithSlowQueries = slowQueriesDuration match {
              case Some(duration) => gqlWithTracing @@ printSlowQueries(duration millis)
              case None           => gqlWithTracing
            }
            interpreter <- gqlWithSlowQueries.interpreter
          } yield BlueprintData(blueprint, blueprint.server.globalResponseTimeout.getOrElse(10000), interpreter)
        }
      } yield dl
    }

}
