package tailcall.gateway.ast

import caliban.GraphQL
import caliban.introspection.adt.{__Directive, __Type, __TypeKind}
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.wrappers.Wrapper
import tailcall.gateway.StepGenerator
import tailcall.gateway.remote.{Remote, RemoteRuntime}
import zio.schema.{DeriveSchema, DynamicValue, Schema}

final case class Orc(nodes: List[Orc.NamedNode]) {
  self =>
  def ++(other: Orc): Orc = Orc(self.nodes ++ other.nodes)

  def toGraphQL: GraphQL[RemoteRuntime] =
    new GraphQL[RemoteRuntime] {
      val schema = new caliban.schema.Schema[RemoteRuntime, Orc] {
        override protected[this] def toType(
          isInput: Boolean,
          isSubscription: Boolean
        ): __Type = __Type(__TypeKind.OBJECT)

        override def resolve(orc: Orc): Step[RemoteRuntime] =
          new StepGenerator(orc).gen
      }

      override protected val schemaBuilder: RootSchemaBuilder[RemoteRuntime] =
        RootSchemaBuilder(
          Option(Operation(schema.toType_(), schema.resolve(self))),
          None,
          None,
          Nil,
          Nil
        )
      override protected val wrappers: List[Wrapper[Any]]            = Nil
      override protected val additionalDirectives: List[__Directive] = Nil
    }
}

object Orc {
  final case class Resolver(remote: Remote[Context] => Remote[OExit])
  object Resolver {
    def make[A](
      f: Remote[Context] => Remote[A]
    )(implicit schema: Schema[A]): Resolver =
      Resolver(context => Remote(OExit.value(f(context).toDynamicValue)))

    def value[A](a: A)(implicit schema: Schema[A]): Resolver =
      Resolver(_ => Remote(OExit.value(a)))

    def ref(name: String): Resolver = Resolver(_ => Remote(OExit.ref(name)))
  }
  final case class Field(name: String, resolver: Resolver)
  final case class NamedNode(name: String, fields: List[Field])

  sealed trait OExit
  object OExit {
    final case class Value(value: DynamicValue) extends OExit
    final case class Ref(name: String)          extends OExit

    def value[A](a: A)(implicit schema: Schema[A]): OExit =
      Value(DynamicValue(a))

    def ref(name: String): OExit = Ref(name)

    implicit lazy val schema: Schema[OExit] = DeriveSchema.gen[OExit]
  }

  def node(name: String)(fields: (String, Resolver)*): NamedNode =
    NamedNode(name, fields.map(field).toList)

  def field(input: (String, Resolver)): Field = Field(input._1, input._2)

  def make(nodes: NamedNode*): Orc = Orc(nodes.toList)

  implicit lazy val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
