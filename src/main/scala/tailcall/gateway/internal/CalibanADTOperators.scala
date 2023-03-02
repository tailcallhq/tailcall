package tailcall.gateway.internal
import caliban.parsing.adt._
object CalibanADTOperators:
  implicit final class DocumentOperator(document: Document):
    def findDefinition(name: String): Option[Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition] =
      document.definitions.collectFirst {
        case d: Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition if d.name == name => d
      }

  implicit final class TypeOperator(ofType: Type):
    def resolveName: String =
      ofType match
        case Type.ListType(ofType, _) => ofType.resolveName
        case Type.NamedType(name, _)  => name
