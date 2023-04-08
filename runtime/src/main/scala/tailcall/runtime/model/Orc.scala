package tailcall.runtime.model

import tailcall.runtime.model.Orc.{Field, FieldSet, Input, LabelledField, Output}
import tailcall.runtime.remote.Remote
import tailcall.runtime.transcoder.Transcoder
import zio.schema.{DeriveSchema, DynamicValue, Schema}
import zio.{Task, ZIO}

/**
 * A scala DSL to create an orchestration specification.
 */
final case class Orc(
  query: Option[String] = None,
  mutation: Option[String] = None,
  subscription: Option[String] = None,
  types: List[Orc.Obj] = Nil,
) {
  self =>
  def toBlueprint: Task[Blueprint] = {
    Transcoder.toBlueprint(self).toEither match {
      case Left(err) => ZIO.fail(new RuntimeException(err))
      case Right(b)  => ZIO.succeed(b)
    }
  }

  def withQuery(name: String): Orc = self.copy(query = Option(name))

  def withInput(spec: (String, List[(String, Field[Input])])*): Orc = withTypes(spec.toList)(FieldSet.InputSet(_))

  def withOutput(spec: (String, List[(String, Field[Output])])*): Orc = withTypes(spec.toList)(FieldSet.OutputSet(_))

  def withTypes[A](spec: List[(String, List[(String, Field[A])])])(f: List[LabelledField[A]] => FieldSet): Orc =
    self.copy(types = self.types ++ spec.map { case (name, fields) =>
      Orc.Obj(name, f(fields.map { case (name, field) => LabelledField(name, field) }))
    })
}

object Orc {
  val empty: Orc = Orc(Option("Query"), Option("Mutation"), None, Nil)

  def apply(spec: (String, FieldSet)*): Orc =
    Orc.empty.copy(types = spec.toList.map { case (name, fields) => Obj(name, fields) })

  final case class LabelledField[A](name: String, field: Field[A])
  final case class Obj(name: String, fields: FieldSet = FieldSet.Empty) {
    def withFields(fields: LabelledField[Output]*): Obj = copy(fields = FieldSet.OutputSet(fields.toList))
    def withInputs(fields: LabelledField[Input]*): Obj  = copy(fields = FieldSet.InputSet(fields.toList))
    def withName(name: String): Obj                     = copy(name = name)
  }

  final case class Input(defaultValue: Option[DynamicValue])
  final case class Output(arguments: List[LabelledField[Input]] = Nil, resolve: Resolver)

  final case class Field[A](ofType: Option[Type], definition: A, annotations: List[FieldUpdateAnnotation]) {
    self =>
    def @@(annotation: FieldUpdateAnnotation): Field[A] = copy(annotations = annotation :: self.annotations)

    def asList: Field[A] = copy(ofType = ofType.map(Type.ListType(_)))

    def asRequired: Field[A] = copy(ofType = ofType.map(Type.NonNull(_)))

    def resolveWith[T](t: T)(implicit s: Schema[T], ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Resolver.fromFunction(_ => Remote(DynamicValue(t)))))

    def resolveWithFunction(f: Remote[DynamicValue] => Remote[DynamicValue])(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Resolver.fromFunction(f)))

    def to(name: String): Field[A] = copy(ofType = Option(Type.NamedType(name)))

    def withArgument(fields: (String, Field[Input])*)(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(arguments = fields.toList.map(f => LabelledField(f._1, f._2))))

    def withDefault[T](t: T)(implicit s: Schema[T], ev: A <:< Input): Field[Input] =
      copy(definition = definition.copy(defaultValue = Option(DynamicValue(t))))
  }

  object Field {
    def input: Field[Input]   = Field(None, Input(None), Nil)
    def output: Field[Output] = Field(None, Output(Nil, Resolver.empty), Nil)
  }

  sealed trait FieldSet
  object FieldSet {
    final case class InputSet(fields: List[LabelledField[Input]])   extends FieldSet
    final case class OutputSet(fields: List[LabelledField[Output]]) extends FieldSet
    case object Empty                                               extends FieldSet

    def apply[A](fields: (String, Field[A])*)(implicit ev: IsField[A]): FieldSet = ev(fields.toList)
  }

  sealed trait IsField[A] {
    def apply(fields: List[(String, Field[A])]): FieldSet
  }

  object IsField {
    implicit case object IsInput extends IsField[Input] {
      override def apply(fields: List[(String, Field[Input])]): FieldSet =
        FieldSet.InputSet(fields.map(f => LabelledField(f._1, f._2)))
    }

    implicit case object IsOutput extends IsField[Output] {
      override def apply(fields: List[(String, Field[Output])]): FieldSet =
        FieldSet.OutputSet(fields.map(f => LabelledField(f._1, f._2)))
    }
  }

  sealed trait Resolver
  object Resolver {
    def fromFunction(f: Remote[DynamicValue] => Remote[DynamicValue]): Resolver = FromFunction(f)
    def empty: Resolver                                                         = Empty
    final case class FromFunction(f: Remote[DynamicValue] => Remote[DynamicValue]) extends Resolver
    case object Empty                                                              extends Resolver
  }

  sealed trait Type
  object Type {
    final case class NonNull(ofType: Type)   extends Type
    final case class NamedType(name: String) extends Type
    final case class ListType(ofType: Type)  extends Type
  }

  implicit val schema: Schema[Orc] = DeriveSchema.gen[Orc]
}
