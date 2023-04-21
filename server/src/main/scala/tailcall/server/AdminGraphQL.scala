package tailcall.server

import caliban.introspection.adt.{__Type, __TypeKind}
import caliban.schema.Annotations.GQLName
import caliban.schema.{GenericSchema, Schema, Step}
import caliban.{ResponseValue, RootResolver, Value}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.lambda.~>
import tailcall.runtime.model.{Blueprint, Digest}
import tailcall.runtime.transcoder.Transcoder
import zio.ZIO
import zio.json.EncoderOps
import zio.query.ZQuery
import zio.schema.DynamicValue

object AdminGraphQL {
  type AdminGraphQLEnv = SchemaRegistry
  object schema extends GenericSchema[AdminGraphQLEnv]
  import schema.auto._

  final case class BlueprintSpec(digest: Digest, source: Blueprint, url: String)
  object BlueprintSpec {
    def apply(digest: Digest, source: Blueprint): BlueprintSpec =
      BlueprintSpec(digest, source, s"/graphql/${digest.hex}")
  }

  @GQLName("Query")
  final case class Query[R, E](blueprints: ZIO[R, E, List[BlueprintSpec]], digests: ZIO[R, E, List[Digest]])

  implicit val lambdaSchema: Schema[Any, DynamicValue ~> DynamicValue] = new Schema[Any, DynamicValue ~> DynamicValue] {
    override def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("Lambda"))

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
    override def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("DynamicValue"))
    override def resolve(value: DynamicValue): Step[Any]                   =
      Step.PureStep(Transcoder.toResponseValue(value).getOrElse(_ => Value.NullValue))
  }

  val graphQL = caliban.graphQL[AdminGraphQLEnv, Query[AdminGraphQLEnv, Throwable], Unit, Unit](RootResolver(Query(
    SchemaRegistry.list(0, Int.MaxValue).map(_.map(blueprint => BlueprintSpec(blueprint.digest, blueprint))),
    SchemaRegistry.digests(0, Int.MaxValue),
  )))
}
