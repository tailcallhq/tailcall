package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.Blueprint
import tailcall.gateway.remote.Remote
import zio.Task
import zio.schema.{DeriveSchema, DynamicValue, Schema}

/**
 * A scala DSL to create an orchestration specification.
 */
final case class Orc(
  query: Option[String] = None,
  mutation: Option[String] = None,
  subscription: Option[String] = None,
  types: List[Orc.Obj] = Nil
) {
  self =>
  def toBlueprint: Task[Blueprint] = OrcCodec.toDocument(self).mapError(new RuntimeException(_))
  def withQuery(name: String): Orc = self.copy(query = Some(name))
  def withType(obj: Orc.Obj*): Orc = self.copy(types = obj.toList ++ types)
}

object Orc {
  final case class LabelledField[A](name: String, field: Field[A])
  final case class Obj(name: String, fields: FieldSet = FieldSet.Empty) {
    def withFields(fields: LabelledField[Output]*): Obj = copy(fields = FieldSet.OutputSet(fields.toList))
    def withInputs(fields: LabelledField[Input]*): Obj  = copy(fields = FieldSet.InputSet(fields.toList))
    def withName(name: String): Obj                     = copy(name = name)
  }

  sealed trait FieldSet
  object FieldSet {
    case object Empty                                               extends FieldSet
    final case class InputSet(fields: List[LabelledField[Input]])   extends FieldSet
    final case class OutputSet(fields: List[LabelledField[Output]]) extends FieldSet
  }

  final case class Input(defaultValue: Option[DynamicValue])
  final case class Output(arguments: List[LabelledField[Input]] = Nil, resolve: Resolver)
  sealed trait Resolver
  object Resolver {
    case object Empty                                                              extends Resolver
    final case class FromFunction(f: Remote[DynamicValue] => Remote[DynamicValue]) extends Resolver

    case object FromParent extends Resolver

    def fromFunction(f: Remote[DynamicValue] => Remote[DynamicValue]): Resolver = FromFunction(f)
    def empty: Resolver                                                         = Empty
    def fromParent: Resolver                                                    = FromParent
  }

  final case class Field[A](ofType: Option[Type], definition: A) {
    self =>
    def to(name: String): Field[A] = copy(ofType = Some(Type.NamedType(name)))

    def toList(name: String): Field[A] = copy(ofType = Some(Type.ListType(Type.NamedType(name))))

    def asRequired: Field[A] = copy(ofType = ofType.map(Type.NonNull))

    def resolveWith[T](t: T)(implicit s: Schema[T], ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Resolver.fromFunction(_ => Remote(DynamicValue(t)))))

    def resolveWithFunction(f: Remote[DynamicValue] => Remote[DynamicValue])(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Resolver.fromFunction(f)))

    def resolveWithParent(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Resolver.fromParent))

    def withDefault[T](t: T)(implicit s: Schema[T], ev: A <:< Input): Field[Input] =
      copy(definition = definition.copy(defaultValue = Some(DynamicValue(t))))

    def withArgument(fields: (String, Field[Input])*)(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(arguments = fields.toList.map(f => LabelledField(f._1, f._2))))
  }

  object Field {
    def input: Field[Input]   = Field(None, Input(None))
    def output: Field[Output] = Field(None, Output(Nil, Resolver.empty))
  }

  sealed trait Type
  object Type {
    final case class NonNull(ofType: Type)   extends Type
    final case class NamedType(name: String) extends Type
    final case class ListType(ofType: Type)  extends Type
  }

  def apply(spec: (String, List[(String, Field[Output])])*): Orc = {
    Orc(
      query = Some("Query"),
      mutation = Some("Mutation"),
      types = spec.toList.map { case (name, fields) =>
        Orc.Obj(name, FieldSet.OutputSet(fields.map { case (name, field) => LabelledField(name, field) }))
      }
    )
  }

  implicit val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
