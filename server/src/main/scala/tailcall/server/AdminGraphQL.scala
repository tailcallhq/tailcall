package tailcall.server

import caliban.introspection.adt.{__Type, __TypeKind}
import caliban.schema.Annotations.GQLName
import caliban.schema.{GenericSchema, Schema, Step}
import caliban.{GraphQL, RootResolver, Value}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.lambda.~>
import tailcall.server.service.BinaryDigest.Digest
import tailcall.server.service.{BinaryDigest, SchemaRegistry}
import zio.ZIO
import zio.schema.DynamicValue

object AdminGraphQL {
  type AdminGraphQLEnv = BinaryDigest with SchemaRegistry
  object schema extends GenericSchema[AdminGraphQLEnv]
  import schema._

  final case class BlueprintSpec(digest: Digest, source: Blueprint, url: String)
  object BlueprintSpec {
    def apply(digest: Digest, source: Blueprint): BlueprintSpec =
      BlueprintSpec(digest, source, s"http://localhost:8080/graphql/${digest.alg.name}/${digest.hex}")
  }

  @GQLName("Query")
  final case class Query[R, E](
    blueprint: Digest => ZIO[R, E, Option[BlueprintSpec]],
    blueprints: ZIO[R, E, List[BlueprintSpec]],
    digests: ZIO[R, E, List[Digest]]
  )

  implicit val lambdaSchema: Schema[Any, DynamicValue ~> DynamicValue] = new Schema[Any, DynamicValue ~> DynamicValue] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("ExecutableFunction"))
    override def resolve(value: DynamicValue ~> DynamicValue): Step[Any]                   =
      Step.PureStep(Value.StringValue("[Binary Data]"))
  }

  implicit val dynamicValueSchema: Schema[Any, DynamicValue] = new Schema[Any, DynamicValue] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("DynamicValue"))
    override def resolve(value: DynamicValue): Step[Any] = Step.PureStep(DynamicValueUtil.toValue(value))
  }

  val graphQL = GraphQL.graphQL[AdminGraphQLEnv, Query[AdminGraphQLEnv, Throwable], Unit, Unit](RootResolver(Query(
    digest =>
      SchemaRegistry.get(digest).map {
        case Some(blueprint) => Option(BlueprintSpec(digest, blueprint))
        case None            => None
      },
    for {
      blueprints <- SchemaRegistry.list(0, Int.MaxValue)
      schemas <- ZIO.foreach(blueprints)(blueprint => BinaryDigest.digest(blueprint).map(BlueprintSpec(_, blueprint)))
    } yield schemas,
    SchemaRegistry.digests(0, Int.MaxValue)
  )))
}
