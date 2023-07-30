package tailcall.server

import caliban.schema.Annotations.GQLName
import caliban.schema.{ArgBuilder, GenericSchema}
import caliban.{GraphQL, RootResolver}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import zio.ZIO

object AdminGraphQL {
  type AdminGraphQLEnv = SchemaRegistry
  private object adminGraphQLEnvSchema extends GenericSchema[AdminGraphQLEnv]
  import adminGraphQLEnvSchema.auto._

  final case class BlueprintSpec(digest: Digest, source: Blueprint, url: String)
  object BlueprintSpec {
    def apply(source: Blueprint): BlueprintSpec = {
      val digest = source.digest
      BlueprintSpec(digest, source, s"/graphql/${digest.hex}")
    }
  }

  @GQLName("Query")
  final case class Query[R, E](
    blueprint: String => ZIO[R, E, Option[BlueprintSpec]],
    blueprints: ZIO[R, E, List[BlueprintSpec]],
    digests: ZIO[R, E, List[Digest]],
  )

  implicit def digestAlgArgBuilder: ArgBuilder[Digest.Algorithm] = { ArgBuilder.gen[Digest.Algorithm] }
  implicit def digestArgBuilder: ArgBuilder[Digest]              = { ArgBuilder.gen[Digest] }

  val graphQL: GraphQL[AdminGraphQLEnv] = caliban
    .graphQL[AdminGraphQLEnv, Query[AdminGraphQLEnv, Throwable], Unit, Unit](RootResolver(Query(
      hex =>
        SchemaRegistry.get(hex).map {
          case Some(blueprint) => Option(BlueprintSpec(blueprint))
          case None            => None
        },
      SchemaRegistry.list(0, Int.MaxValue).map(_.map(BlueprintSpec(_))),
      SchemaRegistry.digests(0, Int.MaxValue),
    )))
}
