package tailcall.server

import caliban.wrappers.ApolloPersistedQueries.{ApolloPersistence, apolloPersistedQueries}
import caliban.wrappers.ApolloTracing.apolloTracing
import caliban.wrappers.Wrappers.printSlowQueries
import caliban.{CalibanError, GraphQL, GraphQLInterpreter}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.Blueprint
import tailcall.runtime.service.{DataLoader, GraphQLGenerator, HttpContext}
import zio._
import zio.http.model.HttpError

object BlueprintDataLoader {
  type InterpreterLoader = DataLoader[GraphQLGenerator, Throwable, String, BlueprintData]

  def default: ZLayer[SchemaRegistry, Nothing, InterpreterLoader] = live(GraphQLConfig.default)

  def live(config: GraphQLConfig): ZLayer[SchemaRegistry, Nothing, InterpreterLoader] =
    ZLayer {
      for {
        registry <- ZIO.service[SchemaRegistry]
        dl       <- DataLoader.one[String] { hex =>
          for {
            maybeBlueprint <- registry.get(hex)
            blueprint      <- ZIO.fromOption(maybeBlueprint)
              .orElseFail(HttpError.BadRequest(s"Blueprint ${hex} has not been published yet."))
            gql            <- blueprint.toGraphQL
            gql            <- ZIO.succeed {
              var _gql: GraphQL[HttpContext with ApolloPersistence] = gql
              if (config.enableTracing) _gql = _gql @@ apolloTracing
              config.slowQueryDuration match {
                case Some(duration) => _gql = _gql @@ printSlowQueries(Duration.fromSeconds(duration))
                case None           => ()
              }
              if (config.persistedQueries) _gql = _gql @@ apolloPersistedQueries
              _gql
            }
            interpreter    <- gql.interpreter
          } yield BlueprintData(blueprint, config.globalResponseTimeout, interpreter)
        }
      } yield dl
    }

  def load(digestId: String): ZIO[GraphQLGenerator with InterpreterLoader, Throwable, BlueprintData] =
    ZIO.serviceWithZIO[InterpreterLoader](_.load(digestId))

  final case class BlueprintData(
    blueprint: Blueprint,
    timeout: Long,
    interpreter: GraphQLInterpreter[HttpContext with ApolloPersistence, CalibanError],
  )

}
