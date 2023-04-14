package tailcall.server

import caliban.introspection.adt.{__Type, __TypeKind}
import caliban.schema.Annotations.GQLName
import caliban.schema.{GenericSchema, Schema, Step}
import caliban.{GraphQL, ResponseValue, RootResolver, Value}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.runtime.remote.~>
import tailcall.runtime.transcoder.Transcoder
import zio.ZIO
import zio.json.EncoderOps
import zio.query.ZQuery
import zio.schema.DynamicValue

object AdminGraphQL {
  type AdminGraphQLEnv = SchemaRegistry
  object schema extends GenericSchema[AdminGraphQLEnv]
  import schema._

  final case class BlueprintSpec(digest: Digest, source: Blueprint, url: String)
  object BlueprintSpec {
    def apply(digest: Digest, source: Blueprint): BlueprintSpec =
      BlueprintSpec(digest, source, s"/graphql/${digest.hex}")
  }

  @GQLName("Query")
  final case class Query[R, E](
    blueprint: Digest => ZIO[R, E, Option[BlueprintSpec]],
    blueprints: ZIO[R, E, List[BlueprintSpec]],
    digests: ZIO[R, E, List[Digest]],
  )

  implicit val lambdaSchema: Schema[Any, DynamicValue ~> DynamicValue] = new Schema[Any, DynamicValue ~> DynamicValue] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("Remote"))

    override def resolve(value: DynamicValue ~> DynamicValue): Step[Any] = {
      Step.QueryStep {
        ZQuery.fromZIO {
          value.compile.toJsonAST flatMap { jsonAst =>
            ResponseValue.responseValueZioJsonDecoder.fromJsonAST(jsonAst)
          } match {
            case Left(value)  => ZIO.fail(new RuntimeException(value))
            case Right(value) => ZIO.succeed(Step.PureStep(value))
          }
        }
      }
    }
  }

  implicit val dynamicValueSchema: Schema[Any, DynamicValue] = new Schema[Any, DynamicValue] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("DynamicValue"))
    override def resolve(value: DynamicValue): Step[Any]                                   =
      Step.PureStep(Transcoder.toResponseValue(value).getOrElse(Value.NullValue))
  }

  val graphQL = GraphQL.graphQL[AdminGraphQLEnv, Query[AdminGraphQLEnv, Throwable], Unit, Unit](RootResolver(Query(
    digest =>
      SchemaRegistry.get(digest).map {
        case Some(blueprint) => Option(BlueprintSpec(digest, blueprint))
        case None            => None
      },
    SchemaRegistry.list(0, Int.MaxValue).map(_.map(blueprint => BlueprintSpec(blueprint.digest, blueprint))),
    SchemaRegistry.digests(0, Int.MaxValue),
  )))
}
