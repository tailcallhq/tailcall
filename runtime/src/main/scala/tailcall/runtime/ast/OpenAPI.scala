package tailcall.runtime.ast

import zio.json.ast.Json

sealed trait OpenAPI

object OpenAPI {
  object V3 {
    sealed abstract case class V0 private (
      openapi: OpenAPI.V3.V0.Version,
      info: OpenAPI.V3.V0.Info,
      servers: List[OpenAPI.V3.V0.Server],
      paths: OpenAPI.V3.V0.Paths,
      components: OpenAPI.V3.V0.Components,
      security: List[OpenAPI.V3.V0.SecurityRequirement],
      tags: List[OpenAPI.V3.V0.Tag],
      externalDocs: Option[OpenAPI.V3.V0.ExternalDocs],
      extensions: Map[OpenAPI.V3.V0.Extensions.Name, Json],
    ) extends OpenAPI

    object V0 {
      def apply(
        openapi: Version,
        info: Info,
        servers: List[Server] = List(),
        paths: Paths,
        components: Components = Components(),
        security: List[SecurityRequirement] = List(),
        tags: List[Tag] = List(),
        externalDocs: Option[ExternalDocs] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      ): OpenAPI.V3.V0 =
        new OpenAPI.V3.V0(
          openapi,
          info,
          if (servers.isEmpty) List(Server("/")) else servers,
          paths,
          components,
          security,
          tags,
          externalDocs,
          extensions,
        ) {}

      sealed trait Version

      final case object V3 extends Version
      final case object V2 extends Version
      final case object V1 extends Version
      final case object V0 extends Version {
        final case object Rc2 extends Version
        final case object Rc1 extends Version
        final case object Rc0 extends Version
      }

      final case class Info(
        title: String,
        description: Option[String] = None,
        termsOfService: Option[String] = None,
        contact: Contact = Contact(),
        license: Option[License] = None,
        version: String,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Contact(
        name: Option[String] = None,
        url: Option[String] = None,
        email: Option[String] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class License(name: String, url: Option[String] = None, extensions: Map[Extensions.Name, Json] = Map())

      final case class Server(
        url: String,
        description: Option[String] = None,
        variables: Map[String, ServerVariable] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      sealed abstract case class ServerVariable private (
        `enum`: List[String],
        default: String,
        description: Option[String],
        extensions: Map[Extensions.Name, Json],
      )

      object ServerVariable {
        def apply(
          `enum`: List[String] = List(),
          default: String,
          description: Option[String] = None,
          extensions: Map[Extensions.Name, Json] = Map(),
        ): ServerVariable = new ServerVariable(`enum`.filter(_ != default), default, description, extensions) {}
      }

      final case class Components(
        schemas: Map[Components.Identifier, Either[Schema, Reference]] = Map(),
        responses: Map[Components.Identifier, Either[Response, Reference]] = Map(),
        parameters: Map[Components.Identifier, Either[Parameter, Reference]] = Map(),
        examples: Map[Components.Identifier, Either[Example, Reference]] = Map(),
        requestBodies: Map[Components.Identifier, Either[RequestBody, Reference]] = Map(),
        headers: Map[Components.Identifier, Either[Header, Reference]] = Map(),
        securitySchemes: Map[Components.Identifier, Either[SecurityScheme, Reference]] = Map(),
        links: Map[Components.Identifier, Either[Link, Reference]] = Map(),
        callbacks: Map[Components.Identifier, Either[Callback, Reference]] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      object Components {
        sealed abstract case class Identifier private (id: String)

        object Identifier {
          val validId = "^[a-zA-Z0-9.\\-_]+$".r

          def fromString(id: String): Option[Identifier] =
            id match {
              case validId() => Some(new Identifier(id) {})
              case _         => None
            }
        }
      }

      final case class Paths(
        pathItems: Map[Paths.Path, PathItem] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      object Paths {
        sealed abstract case class Path private (path: String)

        object Path {
          val validPath = "^\\/".r

          def fromString(path: String): Option[Path] =
            path match {
              case validPath() => Some(new Path(path) {})
              case _           => None
            }
        }
      }

      final case class PathItem(
        ref: Option[String] = None,
        summary: Option[String] = None,
        description: Option[String] = None,
        get: Option[Operation] = None,
        put: Option[Operation] = None,
        post: Option[Operation] = None,
        delete: Option[Operation] = None,
        options: Option[Operation] = None,
        head: Option[Operation] = None,
        patch: Option[Operation] = None,
        trace: Option[Operation] = None,
        servers: List[Server] = List(),
        parameters: Set[Either[Parameter, Reference]] = Set(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Operation(
        tags: List[String] = List(),
        summary: Option[String] = None,
        description: Option[String] = None,
        externalDocs: Option[ExternalDocs] = None,
        operationId: Option[String] = None,
        parameters: Set[Either[Parameter, Reference]] = Set(),
        requestBody: Option[Either[RequestBody, Reference]] = None,
        responses: Responses,
        callbacks: Map[String, Either[Callback, Reference]] = Map(),
        deprecated: Boolean = false,
        security: List[SecurityRequirement] = List(),
        servers: List[Server] = List(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class ExternalDocs(
        description: Option[String] = None,
        url: String,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      sealed trait Parameter {
        type Style <: Parameter.Style
        val name: String
        val description: Option[String]
        val required: Boolean
        val deprecated: Boolean
        val allowEmptyValue: Boolean
        val data: Either[Parameter.WithSchema[Style], Parameter.WithContent]
        val extensions: Map[Extensions.Name, Json]

        override def equals(that: Any): Boolean =
          (this, that) match {
            case (x: Parameter.Path, y: Parameter.Path)     => x.name == y.name
            case (x: Parameter.Query, y: Parameter.Query)   => x.name == y.name
            case (x: Parameter.Header, y: Parameter.Header) => x.name == y.name
            case (x: Parameter.Cookie, y: Parameter.Cookie) => x.name == y.name
            case _                                          => false
          }
      }

      object Parameter {
        sealed trait Style {
          val explode: Boolean
        }

        object Style {
          sealed trait Path   extends Style
          sealed trait Query  extends Style
          sealed trait Header extends Style
          sealed trait Cookie extends Style

          final case class Matrix(explode: Boolean = false)         extends Path
          final case class Label(explode: Boolean = false)          extends Path
          final case class Form(explode: Boolean = true)            extends Query with Cookie
          final case class Simple(explode: Boolean = false)         extends Path with Header
          final case class SpaceDelimited(explode: Boolean = false) extends Query
          final case class PipeDelimited(explode: Boolean = false)  extends Query
          final case class DeepObject(explode: Boolean = false)     extends Query
        }

        sealed abstract case class WithSchema[Style <: Parameter.Style] private (
          style: Style,
          allowReserved: Boolean,
          schema: Either[Schema, Reference],
          examples: Map[String, Either[Example, Reference]],
        )

        object WithSchema {
          def inPath(
            style: Style.Path = Style.Simple(),
            schema: Either[Schema, Reference],
            examples: Map[String, Either[Example, Reference]] = Map(),
          ): WithSchema[Style.Path] = new WithSchema(style, false, schema, examples) {}

          def inQuery(
            style: Style.Query = Style.Form(),
            allowReserved: Boolean = false,
            schema: Either[Schema, Reference],
            examples: Map[String, Either[Example, Reference]] = Map(),
          ): WithSchema[Style.Query] = new WithSchema(style, allowReserved, schema, examples) {}

          def inHeader(
            style: Style.Header = Style.Simple(),
            schema: Either[Schema, Reference],
            examples: Map[String, Either[Example, Reference]] = Map(),
          ): WithSchema[Style.Header] = new WithSchema(style, false, schema, examples) {}

          def inCookie(
            style: Style.Cookie = Style.Form(),
            schema: Either[Schema, Reference],
            examples: Map[String, Either[Example, Reference]] = Map(),
          ): WithSchema[Style.Cookie] = new WithSchema(style, false, schema, examples) {}
        }

        final case class WithContent(mediaType: String, content: MediaType)

        final case class Path(
          name: String,
          description: Option[String] = None,
          deprecated: Boolean = false,
          allowEmptyValue: Boolean = false,
          data: Either[WithSchema[Parameter.Style.Path], WithContent],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends Parameter {
          type Style = Parameter.Style.Path
          val required: Boolean = true
        }

        final case class Query(
          name: String,
          description: Option[String] = None,
          required: Boolean = false,
          deprecated: Boolean = false,
          allowEmptyValue: Boolean = false,
          data: Either[WithSchema[Parameter.Style.Query], WithContent],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends Parameter {
          type Style = Parameter.Style.Query
        }

        final case class Header(
          name: String,
          description: Option[String] = None,
          required: Boolean = false,
          deprecated: Boolean = false,
          allowEmptyValue: Boolean = false,
          data: Either[WithSchema[Parameter.Style.Header], WithContent],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends Parameter {
          type Style = Parameter.Style.Header
        }

        final case class Cookie(
          name: String,
          description: Option[String] = None,
          required: Boolean = false,
          deprecated: Boolean = false,
          allowEmptyValue: Boolean = false,
          data: Either[WithSchema[Parameter.Style.Cookie], WithContent],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends Parameter {
          type Style = Parameter.Style.Cookie
        }
      }

      final case class RequestBody(
        description: Option[String] = None,
        content: Map[String, MediaType],
        required: Boolean = false,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class MediaType(
        schema: Option[Either[Schema, Reference]] = None,
        examples: Map[String, Either[Example, Reference]] = Map(),
        encoding: Map[String, Encoding] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Encoding(
        contentType: Option[String] = None,
        headers: Map[String, Either[Header, Reference]] = Map(),
        style: Parameter.Style.Query = Parameter.Style.Form(),
        allowReserved: Boolean = false,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      sealed abstract case class Responses private (
        default: Option[Either[Response, Reference]],
        httpCodes: Map[Responses.HttpCode, Either[Response, Reference]],
        extensions: Map[Extensions.Name, Json],
      )

      object Responses {
        def apply(
          default: Option[Either[Response, Reference]] = None,
          httpCodes: Map[HttpCode, Either[Response, Reference]] = Map(),
          extensions: Map[Extensions.Name, Json] = Map(),
        ): Option[Responses] =
          if (default.isEmpty && httpCodes.isEmpty) None else Some(new Responses(default, httpCodes, extensions) {})

        sealed abstract case class HttpCode private (code: String)

        object HttpCode {
          val validCode = "^[1-5](?:\\d{2}|XX)$".r

          def fromString(code: String): Option[HttpCode] =
            code match {
              case validCode() => Some(new HttpCode(code) {})
              case _           => None
            }
        }
      }

      final case class Response(
        description: String,
        headers: Map[String, Either[Header, Reference]] = Map(),
        content: Map[String, MediaType] = Map(),
        links: Map[String, Either[Link, Reference]] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Callback(
        expressions: Map[String, PathItem] = Map(),
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Example(
        summary: Option[String] = None,
        description: Option[String] = None,
        value: Option[Json] = None,
        externalValue: Option[String] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Link(
        operation: Option[String] = None,
        parameters: Map[String, Either[Json, String]] = Map(),
        requestBody: Option[Either[Json, String]] = None,
        description: Option[String] = None,
        server: Option[Server] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Header(
        description: Option[String] = None,
        required: Boolean = false,
        deprecated: Boolean = false,
        allowEmptyValue: Boolean = false,
        data: Either[Parameter.WithSchema[Parameter.Style.Header], Parameter.WithContent],
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Tag(
        name: String,
        description: Option[String] = None,
        externalDocs: Option[ExternalDocs] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      final case class Reference(ref: String)

      sealed abstract case class Schema private (
        title: Option[String],
        multipleOf: Option[Int],
        maximum: Option[Int],
        exclusiveMaximum: Boolean,
        minimum: Option[Int],
        exclusiveMinimum: Boolean,
        maxLength: Option[Int],
        minLength: Int,
        pattern: Option[String],
        maxItems: Option[Int],
        minItems: Int,
        uniqueItems: Boolean,
        maxProperties: Option[Int],
        minProperties: Int,
        required: Set[String],
        `enum`: List[Json],
        `type`: Option[Schema.Type],
        not: Option[Either[Schema, Reference]],
        allOf: List[Either[Schema, Reference]],
        oneOf: List[Either[Schema, Reference]],
        anyOf: List[Either[Schema, Reference]],
        items: Option[Either[Schema, Reference]],
        properties: Map[String, Either[Schema, Reference]],
        additionalProperties: Either[Boolean, Either[Schema, Reference]],
        description: Option[String],
        format: Option[String],
        default: Option[Json],
        nullable: Boolean,
        discriminator: Option[Discriminator],
        readOnly: Boolean,
        writeOnly: Boolean,
        example: Option[Json],
        externalDocs: Option[ExternalDocs] = None,
        deprecated: Boolean,
        xml: Option[XML],
        extensions: Map[Extensions.Name, Json],
      )

      object Schema {
        sealed trait Type

        object Type {
          final case object Array   extends Type
          final case object Boolean extends Type
          final case object Integer extends Type
          final case object Number  extends Type
          final case object Object  extends Type
          final case object String  extends Type
        }

        def apply(
          title: Option[String] = None,
          multipleOf: Option[Int] = None,
          maximum: Option[Int] = None,
          exclusiveMaximum: Boolean = false,
          minimum: Option[Int] = None,
          exclusiveMinimum: Boolean = false,
          maxLength: Option[Int] = None,
          minLength: Int = 0,
          pattern: Option[String] = None,
          maxItems: Option[Int] = None,
          minItems: Int = 0,
          uniqueItems: Boolean = false,
          maxProperties: Option[Int] = None,
          minProperties: Int = 0,
          required: Set[String] = Set(),
          `enum`: List[Json] = List(),
          `type`: Option[Schema.Type] = None,
          not: Option[Either[Schema, Reference]] = None,
          allOf: List[Either[Schema, Reference]] = List(),
          oneOf: List[Either[Schema, Reference]] = List(),
          anyOf: List[Either[Schema, Reference]] = List(),
          items: Option[Either[Schema, Reference]] = None,
          properties: Map[String, Either[Schema, Reference]] = Map(),
          additionalProperties: Either[Boolean, Either[Schema, Reference]] = Left(true),
          description: Option[String] = None,
          format: Option[String] = None,
          default: Option[Json] = None,
          nullable: Boolean = false,
          discriminator: Option[Discriminator] = None,
          readOnly: Boolean = false,
          writeOnly: Boolean = false,
          example: Option[Json] = None,
          externalDocs: Option[ExternalDocs] = None,
          deprecated: Boolean = false,
          xml: Option[XML] = None,
          extensions: Map[Extensions.Name, Json] = Map(),
        ): Schema =
          new Schema(
            title,
            multipleOf.filter(_ > 0),
            maximum,
            exclusiveMaximum,
            minimum,
            exclusiveMinimum,
            maxLength.filter(_ >= 0),
            if (minLength >= 0) minLength else 0,
            pattern,
            maxItems.filter(_ >= 0),
            if (minItems >= 0) minItems else 0,
            uniqueItems,
            maxProperties.filter(_ >= 0),
            if (minProperties >= 0) minProperties else 0,
            required,
            `enum`,
            `type`,
            not,
            allOf,
            oneOf,
            anyOf,
            items,
            properties,
            additionalProperties,
            description,
            format,
            default,
            nullable,
            discriminator,
            readOnly,
            writeOnly,
            example,
            externalDocs,
            deprecated,
            xml,
            extensions,
          ) {}
      }

      final case class Discriminator(propertyName: String, mapping: Map[String, String] = Map())

      final case class XML(
        name: Option[String] = None,
        namespace: Option[String] = None,
        prefix: Option[String] = None,
        attribute: Boolean = false,
        wrapped: Boolean = false,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      sealed trait SecurityScheme {
        val description: Option[String]
        val extensions: Map[Extensions.Name, Json]
      }

      object SecurityScheme {
        final case class ApiKey(
          description: Option[String] = None,
          name: String,
          in: ApiKey.Location,
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends SecurityScheme

        object ApiKey {
          sealed trait Location

          object Location {
            final case object Query  extends Location
            final case object Header extends Location
            final case object Cookie extends Location
          }
        }

        final case class Http(
          description: Option[String] = None,
          scheme: Http.Scheme,
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends SecurityScheme

        object Http {
          sealed trait Scheme

          object Scheme {
            final case class Bearer(format: Option[String] = None) extends Scheme
            final case class Other(scheme: String)                 extends Scheme
          }
        }

        final case class OAuth2(
          description: Option[String] = None,
          flows: OAuthFlows,
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends SecurityScheme

        final case class OpenIdConnect(
          description: Option[String] = None,
          openIdConnectUrl: String,
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends SecurityScheme
      }

      final case class OAuthFlows(
        `implicit`: Option[OAuthFlow.Implicit] = None,
        password: Option[OAuthFlow.Password] = None,
        clientCredentials: Option[OAuthFlow.ClientCredentials] = None,
        authorizationCode: Option[OAuthFlow.AuthorizationCode] = None,
        extensions: Map[Extensions.Name, Json] = Map(),
      )

      sealed trait OAuthFlow {
        val refreshUrl: Option[String]
        val scopes: Map[String, String]
        val extensions: Map[Extensions.Name, Json]
      }

      object OAuthFlow {
        final case class Implicit(
          authorizationUrl: String,
          refreshUrl: Option[String] = None,
          scopes: Map[String, String],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends OAuthFlow

        final case class Password(
          tokenUrl: String,
          refreshUrl: Option[String] = None,
          scopes: Map[String, String],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends OAuthFlow

        final case class ClientCredentials(
          tokenUrl: String,
          refreshUrl: Option[String] = None,
          scopes: Map[String, String],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends OAuthFlow

        final case class AuthorizationCode(
          authorizationUrl: String,
          tokenUrl: String,
          refreshUrl: Option[String] = None,
          scopes: Map[String, String],
          extensions: Map[Extensions.Name, Json] = Map(),
        ) extends OAuthFlow
      }

      final case class SecurityRequirement(securitySchemes: Map[String, List[String]] = Map())

      object Extensions {
        sealed abstract case class Name private (name: String)

        object Name {
          val validName = "^x-".r

          def fromString(name: String): Option[Name] =
            name match {
              case validName() => Some(new Name(name) {})
              case _           => None
            }
        }
      }
    }
  }
}
