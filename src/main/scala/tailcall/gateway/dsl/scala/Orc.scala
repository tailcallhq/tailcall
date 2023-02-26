package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.{Context, Document}
import tailcall.gateway.remote.Remote
import zio.schema.{DynamicValue, Schema}

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
  def toDocument: Document         = Orc.toDocument(self)
  def withQuery(name: String): Orc = self.copy(query = Some(name))
  def withType(obj: Orc.Obj*): Orc = self.copy(types = obj.toList ++ types)
}

object Orc {
  final case class Obj(name: String, fields: FieldSet = FieldSet.Empty) {
    self =>

    def withFields[A](fields: Field[A]*)(implicit ev: FieldSetConstructor[A]): Obj = self.copy(fields = ev(fields))
  }

  sealed trait FieldSet
  object FieldSet {
    case object Empty                                          extends FieldSet
    final case class Input(fields: List[Field[Field.Input]])   extends FieldSet
    final case class Output(fields: List[Field[Field.Output]]) extends FieldSet
  }

  final case class Field[A](name: String, ofType: Option[Type], definition: A) {
    self =>
    def as(name: String): Field[A] = copy(ofType = Some(Type.NamedType(name)))

    def asList: Field[A] = copy(ofType = ofType.map(Type.ListType))

    def asRequired: Field[A] = copy(ofType = ofType.map(Type.NonNull))

    def resolveWith[T](t: T)(implicit s: Schema[T], ev: A <:< Field.Output): Field[Field.Output] =
      copy(definition = definition.copy(resolve = Some(_ => Remote(DynamicValue(t)))))

    def withResolver(f: Remote[Context] => Remote[DynamicValue])(implicit ev: A <:< Field.Output): Field[Field.Output] =
      copy(definition = definition.copy(resolve = Some(f)))

    def withDefault[T](t: T)(implicit s: Schema[T], ev: A <:< Field.Input): Field[Field.Input] =
      copy(definition = definition.copy(defaultValue = Some(DynamicValue(t))))

    def withArgument(fields: Field[Field.Input]*)(implicit ev: A <:< Field.Output): Field[Field.Output] =
      copy(definition = definition.copy(arguments = fields.toList))
  }

  object Field {
    sealed trait Definition
    final case class Input(defaultValue: Option[DynamicValue]) extends Definition
    final case class Output(
      arguments: List[Field[Input]] = Nil,
      resolve: Option[Remote[Context] => Remote[DynamicValue]]
    ) extends Definition

    def input(name: String): Field[Input]   = Field(name, None, Input(None))
    def output(name: String): Field[Output] = Field(name, None, Output(Nil, None))
  }

  sealed trait Type
  object Type {
    final case class NonNull(ofType: Type)   extends Type
    final case class NamedType(name: String) extends Type
    final case class ListType(ofType: Type)  extends Type
  }

  def empty: Orc = Orc()

  sealed trait FieldSetConstructor[A] {
    def apply(a: Seq[Field[A]]): FieldSet
  }
  object FieldSetConstructor          {
    implicit object Input  extends FieldSetConstructor[Field.Input]  {
      override def apply(a: Seq[Field[Field.Input]]): FieldSet = FieldSet.Input(a.toList)
    }
    implicit object Output extends FieldSetConstructor[Field.Output] {
      override def apply(a: Seq[Field[Field.Output]]): FieldSet = FieldSet.Output(a.toList)
    }
  }

  /// ---- ---- ----

  // TODO: add unit tests
  private def toType(t: Type, isNull: Boolean = true): Document.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, nonNull)
      case Type.NamedType(name)  => Document.NamedType(name, nonNull)
      case Type.ListType(ofType) => Document.ListType(toType(ofType, nonNull), nonNull)
    }
  }

  private def toDefinition(field: Field[Field.Input]): Document.InputValueDefinition =
    Document.InputValueDefinition(field.name, toType(field.ofType.getOrElse(???)), field.definition.defaultValue)

  private def toDefinition(field: Field[Field.Output]): Document.FieldDefinition =
    Document.FieldDefinition(
      name = field.name,
      ofType = toType(field.ofType.getOrElse(???)),
      args = field.definition.arguments.map(toDefinition),
      resolver = field.definition.resolve.getOrElse(???)
    )

  private def toDocument(o: Orc): Document = {
    val schemaDefinition = Document
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    val objectDefinitions: List[Document.Definition] = o.types.collect {
      case Orc.Obj(name, FieldSet.Input(fields))  => Document.InputObjectTypeDefinition(name, fields.map(toDefinition))
      case Orc.Obj(name, FieldSet.Output(fields)) => Document.ObjectTypeDefinition(name, fields.map(toDefinition))
    }

    Document(schemaDefinition :: objectDefinitions)
  }
}
