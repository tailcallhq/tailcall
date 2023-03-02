package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.Document
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
):
  self =>
  def toDocument: Task[Document]   = OrcCodec.toDocument(self).mapError(new RuntimeException(_))
  def withQuery(name: String): Orc = self.copy(query = Some(name))
  def withType(obj: Orc.Obj*): Orc = self.copy(types = obj.toList ++ types)

object Orc:
  final case class LabelledField[A](name: String, field: Field[A])
  final case class Obj(name: String, fields: FieldSet = FieldSet.Empty):
    def withFields(fields: LabelledField[Output]*): Obj = copy(fields = FieldSet.OutputSet(fields.toList))
    def withInputs(fields: LabelledField[Input]*): Obj  = copy(fields = FieldSet.InputSet(fields.toList))
    def withName(name: String): Obj                     = copy(name = name)

  sealed trait FieldSet
  object FieldSet:
    case object Empty                                               extends FieldSet
    final case class InputSet(fields: List[LabelledField[Input]])   extends FieldSet
    final case class OutputSet(fields: List[LabelledField[Output]]) extends FieldSet

  final case class Input(defaultValue: Option[DynamicValue])
  final case class Output(
    arguments: List[LabelledField[Input]] = Nil,
    resolve: Option[Remote[DynamicValue] => Remote[DynamicValue]]
  )

  final case class Field[A](ofType: Option[Type], definition: A):
    self =>
    def as(name: String): Field[A] = copy(ofType = Some(Type.NamedType(name)))

    def asList: Field[A] = copy(ofType = ofType.map(Type.ListType(_)))

    def asList(name: String): Field[A] = copy(ofType = Some(Type.ListType(Type.NamedType(name))))

    def asRequired: Field[A] = copy(ofType = ofType.map(Type.NonNull(_)))

    def resolveWith[T](t: T)(implicit s: Schema[T], ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Some(_ => Remote(DynamicValue(t)))))

    def withResolver(f: Remote[DynamicValue] => Remote[DynamicValue])(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Some(f)))

    def withDefault[T](t: T)(implicit s: Schema[T], ev: A <:< Input): Field[Input] =
      copy(definition = definition.copy(defaultValue = Some(DynamicValue(t))))

    def withArgument(fields: (String, Field[Input])*)(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(arguments = fields.toList.map(f => LabelledField(f._1, f._2))))

  object Field:
    def input: Field[Input]   = Field(None, Input(None))
    def output: Field[Output] = Field(None, Output(Nil, None))

  sealed trait Type
  object Type:
    final case class NonNull(ofType: Type)   extends Type
    final case class NamedType(name: String) extends Type
    final case class ListType(ofType: Type)  extends Type

  def apply(spec: (String, List[(String, Field[Output])])*): Orc =
    Orc(
      query = Some("Query"),
      mutation = Some("Mutation"),
      types = spec.toList.map { case (name, fields) =>
        Orc.Obj(name, FieldSet.OutputSet(fields.map { case (name, field) => LabelledField(name, field) }))
      }
    )

  implicit val schema: Schema[Orc] = DeriveSchema.gen[Orc]
