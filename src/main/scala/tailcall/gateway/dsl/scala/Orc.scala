package tailcall.gateway.dsl.scala

import tailcall.gateway.ast.{Context, Orchestration}
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue

/**
 * A scala DSL to create an orchestration specification.
 */
object Orc {
  type RemoteFunction = Remote[Context] => Remote[DynamicValue]
  type ReturnType     = Orchestration.Type
  type Field          = (ReturnType, RemoteFunction)
  type LabeledField   = (String, Field)

  def as(name: String)(func: Remote[Context] => Remote[DynamicValue]): Field =
    (Orchestration.NamedType(name, true), func)

  def asList(name: String)(func: Remote[Context] => Remote[DynamicValue]): Field =
    (Orchestration.ListType(Orchestration.NamedType(name, true), true), func)

  def obj(spec: (String, List[LabeledField])*): Orchestration =
    Orchestration(spec.toList.map { case (name, fields) =>
      Orchestration.ObjectTypeDefinition(
        name,
        fields.map { case (name, (returnType, func)) => Orchestration.FieldDefinition(name, List(), returnType, func) }
      )
    })
}
