package tailcall.runtime.ast

import caliban.GraphQL
import tailcall.runtime.lambda.Expression.Logical.Unary
import tailcall.runtime.lambda.Expression.{Dict, Logical, Math, Opt, Unsafe}
import tailcall.runtime.lambda.{Expression, ~>}
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.service.GraphQLGenerator
import zio.ZIO
import zio.json.{JsonCodec, JsonDecoder, JsonEncoder}
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
  def endpoints: List[Endpoint]                  = {
    def find(expr: Expression): List[Endpoint] = {
      expr match {
        case Expression.Identity                    => Nil
        case Expression.Defer(value)                => find(value)
        case Expression.EqualTo(left, right, _)     => find(left) ++ find(right)
        case Expression.FunctionDef(_, body, input) => find(body) ++ find(input)
        case Expression.Immediate(value)            => find(value)
        case Expression.Literal(_, _)               => Nil
        case Expression.Logical(operation)          => operation match {
            case Logical.Binary(_, left, right)  => find(left) ++ find(right)
            case Logical.Unary(value, operation) => operation match {
                case Unary.Diverge(isTrue, isFalse) => find(value) ++ find(isTrue) ++ find(isFalse)
                case Unary.Not                      => find(value)
              }
          }
        case Expression.Lookup(_)                   => Nil
        case Expression.Math(operation, _)          => operation match {
            case Math.Binary(_, left, right) => find(left) ++ find(right)
            case Math.Unary(_, value)        => find(value)
          }
        case Expression.Pipe(left, right)           => find(left) ++ find(right)
        case Expression.Unsafe(operation)           => operation match {
            case Unsafe.Die(_)                 => Nil
            case Unsafe.Debug(_)               => Nil
            case Unsafe.EndpointCall(endpoint) => List(endpoint)
          }
        case Expression.Dynamic(_)                  => Nil
        case Expression.Dict(operation)             => operation match {
            case Dict.Get(key, map)        => find(key) ++ find(map)
            case Dict.Put(key, value, map) => find(key) ++ find(value) ++ find(map)
          }
        case Expression.Opt(operation)              => operation match {
            case Opt.IsSome                  => Nil
            case Opt.IsNone                  => Nil
            case Opt.Fold(value, none, some) => find(value) ++ find(none) ++ find(some)
            case Opt.Apply(value)            => value match {
                case Some(value) => find(value)
                case None        => Nil
              }
          }
      }
    }

    definitions.collect { case Blueprint.ObjectTypeDefinition(_, fields, _) => fields }.flatten
      .flatMap(_.resolver.toList.map(_.compile)).flatMap(find)
  }
}

object Blueprint {
  // TODO: create a common type for Object
  // TODO: drop non-null fields
  // TODO: create a common type for input and field use phantom types

  sealed trait Definition

  final case class ObjectTypeDefinition(name: String, fields: List[FieldDefinition], description: Option[String] = None)
      extends Definition {
    def toInput: InputObjectTypeDefinition =
      InputObjectTypeDefinition(name = name, fields = fields.map(_.toInput(None)), description = description)
  }

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
  )

  final case class FieldDefinition(
    name: String,
    args: List[InputFieldDefinition] = Nil,
    ofType: Type,
    resolver: Option[DynamicValue ~> DynamicValue] = None,
    directives: List[Directive] = Nil,
    description: Option[String] = None,
  ) {
    def toInput(defaultValue: Option[DynamicValue]): InputFieldDefinition =
      InputFieldDefinition(name, ofType, defaultValue)
  }

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
  }

  final case class NamedType(name: String, nonNull: Boolean) extends Type

  final case class ListType(ofType: Type, nonNull: Boolean) extends Type

  implicit val schema: Schema[Blueprint] = DeriveSchema.gen[Blueprint]

  val codec: JsonCodec[Blueprint]              = zio.schema.codec.JsonCodec.jsonCodec(schema)
  implicit val jsonCodec: JsonCodec[Blueprint] = zio.schema.codec.JsonCodec.jsonCodec(schema)
  implicit val objectTypeDefinitionJsonCodec: JsonCodec[ObjectTypeDefinition] = zio.schema.codec.JsonCodec
    .jsonCodec(DeriveSchema.gen[ObjectTypeDefinition])
  implicit val encoder: JsonEncoder[Blueprint]                                = codec.encoder
  implicit val decoder: JsonDecoder[Blueprint]                                = codec.decoder
  def decode(bytes: CharSequence): Either[String, Blueprint]                  = codec.decodeJson(bytes)
  def encode(value: Blueprint): CharSequence                                  = codec.encodeJson(value, None)

  def empty: Blueprint = Blueprint()
}
