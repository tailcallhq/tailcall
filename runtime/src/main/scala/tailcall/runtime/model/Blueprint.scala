package tailcall.runtime.model

import caliban.GraphQL
import tailcall.runtime.lambda.{Expression, ~>}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.GraphQLGenerator
import zio.ZIO
import zio.json.JsonCodec
import zio.schema.{DeriveSchema, DynamicValue, Schema}

import scala.annotation.tailrec

/**
 * Document is an intermediate representation of a GraphQL
 * document. It has two features — 1. It is serializable and
 * 2. It has logic to resolve fields into actual values.
 *
 * IMPORTANT: we should keep this as close to Caliban's AST
 * as much as possible. The idea is that sometimes we might
 * need some changes in Caliban's AST, for eg: we need to
 * generate a ZIO Schema of the Caliban AST. This is
 * currently not possible because the case classes are not
 * final. Instead of opening a PR in Caliban, we can just
 * make the changes here and then use the modified AST. The
 * other reason is that our IR design isn't very clearly
 * thought out. So we will use Document as a playground to
 * try out different IRs. Document supports each and every
 * feature that GraphQL has to offer so we keep it until IR
 * is clearly defined. Once the IR is ready we will directly
 * compile IR to Caliban's Step ADT.
 */
final case class Blueprint(definitions: List[Blueprint.Definition] = Nil) {
  self =>
  def digest: Digest                                                     = Digest.fromBlueprint(self)
  def toGraphQL: ZIO[GraphQLGenerator, Nothing, GraphQL[HttpDataLoader]] = GraphQLGenerator.toGraphQL(self)
  def schema: Option[Blueprint.SchemaDefinition] = definitions.collectFirst { case s: Blueprint.SchemaDefinition => s }
  def resolversMap: Map[String, Map[String, Option[Expression]]] =
    definitions.collect { case r: Blueprint.ObjectTypeDefinition =>
      (r.name, r.fields.map(field => (field.name, field.resolver.map(_.compile))).toMap)
    }.toMap

  def endpoints: List[Endpoint] =
    for {
      fields     <- definitions.collect { case Blueprint.ObjectTypeDefinition(_, fields, _) => fields }
      definition <- fields
      resolver   <- definition.resolver.toList.map(_.compile)
      endpoint   <- resolver.collect { case Expression.Unsafe(Expression.Unsafe.EndpointCall(endpoint)) => endpoint }
    } yield endpoint
}

object Blueprint {
  implicit val schema: Schema[Blueprint]   = DeriveSchema.gen[Blueprint]
  implicit val codec: JsonCodec[Blueprint] = zio.schema.codec.JsonCodec.jsonCodec(schema)

  def decode(bytes: CharSequence): Either[String, Blueprint] = codec.decodeJson(bytes)

  def empty: Blueprint = Blueprint()

  def encode(value: Blueprint): CharSequence = codec.encodeJson(value, None)

  sealed trait Definition

  final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition], description: Option[String] = None)
      extends Definition

  final case class InputObjectTypeDefinition(
    name: String,
    fields: List[InputFieldDefinition],
    description: Option[String] = None,
  ) extends Definition

  final case class SchemaDefinition(
    query: Option[String] = None,
    mutation: Option[String] = None,
    subscription: Option[String] = None,
    directives: List[Directive] = Nil,
  ) extends Definition

  final case class InputFieldDefinition(
    name: String,
    ofType: Type,
    defaultValue: Option[DynamicValue],
    description: Option[String] = None,
    annotations: List[ModifyField] = Nil,
  )

  final case class FieldDefinition(
    name: String,
    args: List[InputFieldDefinition] = Nil,
    ofType: Type,
    resolver: Option[DynamicValue ~> DynamicValue] = None,
    directives: List[Directive] = Nil,
    description: Option[String] = None,
  )

  final case class Directive(name: String, arguments: Map[String, DynamicValue] = Map.empty, index: Int = 0)

  final case class ScalarTypeDefinition(
    name: String,
    directive: List[Directive] = Nil,
    description: Option[String] = None,
  ) extends Definition

  sealed trait Type {
    self =>
    @tailrec
    final def defaultName: String =
      self match {
        case NamedType(name, _)  => name
        case ListType(ofType, _) => ofType.defaultName
      }

    final def withName(name: String): Type =
      self match {
        case NamedType(_, nonNull)     => NamedType(name, nonNull)
        case ListType(ofType, nonNull) => ListType(ofType.withName(name), nonNull)
      }

    final def render: String = {
      def renderNonNull(tpe: Type): String =
        tpe match {
          case NamedType(name, true)   => s"$name!"
          case ListType(ofType, true)  => s"[${renderNonNull(ofType)}]!"
          case NamedType(name, false)  => name
          case ListType(ofType, false) => s"[${renderNonNull(ofType)}]"
        }
      renderNonNull(self)
    }
  }

  final case class NamedType(name: String, nonNull: Boolean) extends Type

  final case class ListType(ofType: Type, nonNull: Boolean) extends Type

}
