package tailcall.gateway.internal
import caliban.parsing.adt._
object DocumentOperators {
  implicit final class DocumentOperator(document: Document) {
    def findDefinition(
      name: String,
    ): Option[Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition] = document
      .definitions
      .collectFirst {
        case d: Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition
            if d.name == name =>
          d
      }
  }
}
