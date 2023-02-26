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
  type LabelledField[A] = (String, Field[A])
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
  final case class Output(
    arguments: List[LabelledField[Input]] = Nil,
    resolve: Option[Remote[Context] => Remote[DynamicValue]]
  )

  final case class Field[A](ofType: Option[Type], definition: A) {
    self =>
    def as(name: String): Field[A] = copy(ofType = Some(Type.NamedType(name)))

    def asList: Field[A] = copy(ofType = ofType.map(Type.ListType))

    def asList(name: String): Field[A] = copy(ofType = Some(Type.ListType(Type.NamedType(name))))

    def asRequired: Field[A] = copy(ofType = ofType.map(Type.NonNull))

    def resolveWith[T](t: T)(implicit s: Schema[T], ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Some(_ => Remote(DynamicValue(t)))))

    def withResolver(f: Remote[Context] => Remote[DynamicValue])(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(resolve = Some(f)))

    def withDefault[T](t: T)(implicit s: Schema[T], ev: A <:< Input): Field[Input] =
      copy(definition = definition.copy(defaultValue = Some(DynamicValue(t))))

    def withArgument(fields: LabelledField[Input]*)(implicit ev: A <:< Output): Field[Output] =
      copy(definition = definition.copy(arguments = fields.toList))
  }

  object Field {
    def input: Field[Input]   = Field(None, Input(None))
    def output: Field[Output] = Field(None, Output(Nil, None))
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
      types = spec.toList.map { case (name, fields) => Orc.Obj(name, FieldSet.OutputSet(fields.toList)) }
    )
  }

  // TODO: add unit tests
  private def toType(t: Type, isNull: Boolean = true): Document.Type = {
    val nonNull = !isNull
    t match {
      case Type.NonNull(ofType)  => toType(ofType, nonNull)
      case Type.NamedType(name)  => Document.NamedType(name, nonNull)
      case Type.ListType(ofType) => Document.ListType(toType(ofType, nonNull), nonNull)
    }
  }

  private def toDefinition(field: LabelledField[Input]): Document.InputValueDefinition =
    Document.InputValueDefinition(field._1, toType(field._2.ofType.getOrElse(???)), field._2.definition.defaultValue)

  private def toDefinition(field: LabelledField[Output]): Document.FieldDefinition =
    Document.FieldDefinition(
      name = field._1,
      ofType = toType(field._2.ofType.getOrElse(???)),
      args = field._2.definition.arguments.map(toDefinition),
      resolver = field._2.definition.resolve.getOrElse(???)
    )

  private def toDocument(o: Orc): Document = {
    val schemaDefinition = Document
      .SchemaDefinition(query = o.query, mutation = o.mutation, subscription = o.subscription)

    val objectDefinitions: List[Document.Definition] = o.types.collect {
      case Orc.Obj(name, FieldSet.InputSet(fields))  => Document
          .InputObjectTypeDefinition(name, fields.map(toDefinition))
      case Orc.Obj(name, FieldSet.OutputSet(fields)) => Document.ObjectTypeDefinition(name, fields.map(toDefinition))
    }

    Document(schemaDefinition :: objectDefinitions)
  }
}
