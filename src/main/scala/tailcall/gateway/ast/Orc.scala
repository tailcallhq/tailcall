package tailcall.gateway.ast

import caliban.GraphQL
import caliban.introspection.adt.{__Directive, __Type, __TypeKind}
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.wrappers.Wrapper
import tailcall.gateway.StepGenerator
import tailcall.gateway.lambda.{LambdaRuntime, ~>}
import zio.schema.{DeriveSchema, DynamicValue, Schema}

sealed trait Orc {
  self =>
  def toGraphQL: GraphQL[LambdaRuntime] =
    new GraphQL[LambdaRuntime] {
      val schema = new caliban.schema.Schema[LambdaRuntime, Orc] {
        override protected[this] def toType(isInput: Boolean, isSubscription: Boolean): __Type =
          __Type(__TypeKind.OBJECT)

        override def resolve(orc: Orc): Step[LambdaRuntime] = new StepGenerator(orc).gen
      }

      override protected val schemaBuilder: RootSchemaBuilder[LambdaRuntime] =
        RootSchemaBuilder(Option(Operation(schema.toType_(), schema.resolve(self))), None, None, Nil, Nil)
      override protected val wrappers: List[Wrapper[Any]]                    = Nil
      override protected val additionalDirectives: List[__Directive]         = Nil
    }
}

object Orc {
  final case class OrcValue(dynamicValue: DynamicValue)              extends Orc
  final case class OrcObject(name: String, fields: Map[String, Orc]) extends Orc
  final case class OrcList(values: List[Orc])                        extends Orc
  final case class OrcFunction(fun: Context ~> Orc)                  extends Orc
  final case class OrcRef(name: String)                              extends Orc

  def value[A](a: A)(implicit schema: Schema[A]): Orc = OrcValue(schema.toDynamic(a))

  def obj(name: String)(fields: (String, Orc)*): Orc = OrcObject(name, fields.toMap)

  def list(values: Orc*): Orc = OrcList(values.toList)

  def function(fun: Context ~> Orc): Orc = OrcFunction(fun)

  def ref(ref: String): Orc = OrcRef(ref)

  implicit lazy val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
