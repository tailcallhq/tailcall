package tailcall.server

import caliban.wrappers.ApolloTracing.apolloTracing
import caliban.wrappers.Wrappers.printSlowQueries
import caliban.{CalibanError, GraphQLInterpreter}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.runtime.service.{DataLoader, GraphQLGenerator, HttpContext}
import zio._
import zio.http.HttpError

object BlueprintDataLoader {
  type InterpreterLoader = DataLoader[GraphQLGenerator, Throwable, String, BlueprintData]

  def default: ZLayer[SchemaRegistry, Nothing, InterpreterLoader] = live(GraphQLConfig.default)

  def live(config: GraphQLConfig): ZLayer[SchemaRegistry, Nothing, InterpreterLoader] =
    ZLayer {
      for {
        registry <- ZIO.service[SchemaRegistry]
        dl       <- DataLoader.one[String] { digestId =>
          for {
            maybeBlueprint <- registry.get(Digest.fromHex(digestId))
            blueprint      <- ZIO.fromOption(maybeBlueprint)
              .orElse(ZIO.fail(HttpError.BadRequest(s"Blueprint ${digestId} has not been published yet.")))
            gql            <- blueprint.toGraphQL
            gqlWithTracing     = if (config.enableTracing) gql @@ apolloTracing else gql
            gqlWithSlowQueries = config.slowQueryDuration match {
              case Some(duration) => gqlWithTracing @@ printSlowQueries(duration)
              case None           => gqlWithTracing
            }
            interpreter <- gqlWithSlowQueries.interpreter
          } yield BlueprintData(blueprint, config.globalResponseTimeout.toSeconds, interpreter)
        }
      } yield dl
    }

  def load(digestId: String): ZIO[GraphQLGenerator with InterpreterLoader, Throwable, BlueprintData] =
    ZIO.serviceWithZIO[InterpreterLoader](_.load(digestId))

  final case class BlueprintData(
    blueprint: Blueprint,
    timeout: Long,
    interpreter: GraphQLInterpreter[HttpContext, CalibanError],
  )

}
