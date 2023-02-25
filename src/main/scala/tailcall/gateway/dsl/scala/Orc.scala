package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.{Context, Document}
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

/**
 * A scala DSL to create an orchestration specification.
 */
object Orc {
  type RemoteFunction = Remote[Context] => Remote[DynamicValue]
  type ReturnType     = Document.Type
  type Field          = (ReturnType, RemoteFunction)
  type LabeledField   = (String, Field)

  def as(name: String)(func: Remote[Context] => Remote[DynamicValue]): Field = (Document.NamedType(name, true), func)

  def asList(name: String)(func: Remote[Context] => Remote[DynamicValue]): Field =
    (Document.ListType(Document.NamedType(name, true), true), func)

  def obj(spec: (String, List[LabeledField])*): Document =
    Document(spec.toList.map { case (name, fields) =>
      Document.ObjectTypeDefinition(
        name,
        fields.map { case (name, (returnType, func)) => Document.FieldDefinition(name, List(), returnType, func) }
      )
    })
}
