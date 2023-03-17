package tailcall.server

import caliban._
import caliban.introspection.adt.{__Type, __TypeKind}
import caliban.schema.Annotations.GQLName
import caliban.schema.{ArgBuilder, GenericSchema, Schema, Step}
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.dsl.json.service.ConfigBlueprint
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.lambda.~>
import tailcall.server.service.BinaryDigest.Digest
import tailcall.server.service.{BinaryDigest, SchemaRegistry}
import zio.ZIO
import zio.json.yaml._
import zio.json.{DecoderOps, EncoderOps}
import zio.query.ZQuery
import zio.schema.{DeriveSchema, DynamicValue}

object AdminGraphQL {
  type AdminGraphQLEnv = SchemaRegistry with BinaryDigest with ConfigBlueprint

  final case class BlueprintSpec(digest: Digest, source: Blueprint, url: String)
  object BlueprintSpec {
    def apply(digest: Digest, source: Blueprint): BlueprintSpec =
      BlueprintSpec(digest, source, s"/graphql/${digest.alg.name}/${digest.hex}")
  }

  @GQLName("Query")
  final case class Query(
    blueprint: Digest => ZIO[SchemaRegistry, Throwable, Option[BlueprintSpec]],
    blueprints: ZIO[BinaryDigest with SchemaRegistry, Throwable, List[BlueprintSpec]],
    digests: ZIO[BinaryDigest with SchemaRegistry, Throwable, List[Digest]]
  )

  @GQLName("Mutation")
  final case class Mutation(
    registerBlueprint: DynamicValue => ZIO[SchemaRegistry, Throwable, BlueprintSpec],
    registerYaml: String => ZIO[SchemaRegistry with ConfigBlueprint, Throwable, BlueprintSpec],
    registerJson: String => ZIO[SchemaRegistry with ConfigBlueprint, Throwable, BlueprintSpec]
  )

  implicit val dvArgbuilder: ArgBuilder[DynamicValue] = new ArgBuilder[DynamicValue] {
    override def build(value: InputValue): Either[Nothing, DynamicValue] = Right(DynamicValueUtil.fromInputValue(value))
  }

  implicit val lambdaSchema: Schema[Any, DynamicValue ~> DynamicValue] = new Schema[Any, DynamicValue ~> DynamicValue] {
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
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
    override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
      __Type(kind = __TypeKind.SCALAR, name = Some("DynamicValue"))
    override def resolve(value: DynamicValue): Step[Any] = Step.PureStep(DynamicValueUtil.toValue(value))
  }

  final case class Temp(value: Blueprint)
  object Temp {
    implicit val schema: zio.schema.Schema[Temp] = DeriveSchema.gen[Temp]
  }
  object schema extends GenericSchema[AdminGraphQLEnv]

  import schema._

  val graphQL = GraphQL.graphQL(RootResolver(
    Query(
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
    ),
    Mutation(
      blueprint =>
        for {
          blueprint <- ZIO.fromOption(DynamicValueUtil.toTyped[Temp](blueprint).map(_.value))
            .mapError(_ => new RuntimeException("Blueprint must be a string"))
          digest    <- SchemaRegistry.add(blueprint)

        } yield BlueprintSpec(digest, blueprint),
      yaml =>
        for {
          config    <- ZIO.fromEither(yaml.fromYaml[Config]).mapError(new RuntimeException(_))
          blueprint <- ConfigBlueprint.toBlueprint(config)
          digest    <- SchemaRegistry.add(blueprint)
        } yield BlueprintSpec(digest, blueprint),
      json =>
        for {
          config    <- ZIO.fromEither(json.fromJson[Config]).mapError(new RuntimeException(_))
          blueprint <- ConfigBlueprint.toBlueprint(config)
          digest    <- SchemaRegistry.add(blueprint)
        } yield BlueprintSpec(digest, blueprint)
    )
  ))
}
