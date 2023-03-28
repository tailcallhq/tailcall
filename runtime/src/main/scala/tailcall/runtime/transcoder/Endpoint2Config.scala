package tailcall.runtime.transcoder

import tailcall.runtime.ast.{Endpoint, TSchema}
import tailcall.runtime.dsl.Config
import tailcall.runtime.dsl.Config.Step.Http
import tailcall.runtime.dsl.Config.{GraphQL, RootSchema, Server}
import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator

import java.net.URL
import java.util.concurrent.atomic.AtomicInteger

trait Endpoint2Config {
  final def toConfig(endpoint: Endpoint, nameGen: NameGenerator = NameGenerator.prefixOnly): TValid[String, Config] =
    Endpoint2Config.Live(nameGen).toConfig(endpoint)
}

object Endpoint2Config {

  final case class Live(nameGen: NameGenerator) {
    def getTypeName(schema: TSchema): String = nameGen.gen("Type", schema)

    def toConfig(endpoint: Endpoint): TValid[String, Config] =
      for {
        baseURL <- toBaseURL(endpoint)
        graphQL <- toGraphQL(endpoint)
      } yield Config(server = Server(baseURL = Option(baseURL)), graphQL = graphQL)

    private def toBaseURL(endpoint: Endpoint): TValid[String, URL] = {
      val urlString = endpoint.address.port match {
        case -1 | 80 | 443 => endpoint.scheme.name + "://" + endpoint.address.host
        case _             => endpoint.scheme.name + "://" + endpoint.address.host + ":" + endpoint.address.port
      }
      try TValid.succeed(new URL(urlString))
      catch { case _: Throwable => TValid.fail(s"Invalid URL:  ${urlString}") }
    }

    private def toConfigField(schema: TSchema, isRequired: Boolean, isList: Boolean): Config.Field = {
      schema match {
        case TSchema.Obj(_)           => Config
            .Field(typeOf = getTypeName(schema), isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Arr(schema)      => toConfigField(schema, isRequired, isList = true)
        case TSchema.Optional(schema) => toConfigField(schema, isRequired = false, isList = isList)
        case TSchema.String => Config.Field(typeOf = "String", isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Int    => Config.Field(typeOf = "Int", isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Boolean => Config
            .Field(typeOf = "Boolean", isRequired = Option(isRequired), isList = Option(isList))
      }
    }

    private def toFields(fields: List[TSchema.Field]): List[(String, Config.Field)] = {
      fields.map(field => field.name -> toConfigField(field.schema, isRequired = true, isList = false))
    }

    private def toGraphQL(endpoint: Endpoint): TValid[String, Config.GraphQL] = {
      for {
        graphQL <- if (endpoint.method == Method.GET) toGraphQLQuery(endpoint) else toGraphQLMutation(endpoint)
      } yield graphQL
    }

    private def toGraphQLMutation(endpoint: Endpoint): TValid[String, Config.GraphQL] = ???

    private def toGraphQLQuery(endpoint: Endpoint): TValid[String, Config.GraphQL] =
      TValid.succeed {
        val types = toTypes(endpoint) :+ ("Query" -> toQueryField(endpoint))
        GraphQL(
          schema = RootSchema(query = Option("Query")),
          types = types.map { case (key, value) => key -> value.toMap }.toMap,
        )
      }

    private def toQueryField(endpoint: Endpoint): List[(String, Config.Field)] = {
      endpoint.output.toList.map(schema =>
        nameGen.gen("field", schema) -> toConfigField(schema, isRequired = true, isList = false)
          .withSteps(List(Http.fromEndpoint(endpoint).withOutput(None))).compress
      )
    }

    private def toTypes(endpoint: Endpoint): List[(String, List[(String, Config.Field)])] = {
      endpoint.output match {
        case Some(schema) => toTypes(schema, isRequired = true, isList = false)
        case None         => Nil
      }
    }

    private def toTypes(
      schema: TSchema,
      isRequired: Boolean,
      isList: Boolean,
    ): List[(String, List[(String, Config.Field)])] = {
      schema match {
        case TSchema.Obj(fields)      => List(getTypeName(schema) -> toFields(fields))
        case TSchema.Arr(item)        => toTypes(item, isRequired, isList = true)
        case TSchema.Optional(schema) => toTypes(schema, isRequired = false, isList = isList)
        case TSchema.String           => Nil
        case TSchema.Int              => Nil
        case TSchema.Boolean          => Nil
      }
    }
  }

  trait NameGenerator {
    final private var cache                                = Map.empty[TSchema, String]
    final def gen(prefix: String, schema: TSchema): String = {
      val name = make(prefix, schema)
      cache = cache.updated(schema, name)
      name
    }
    def make(prefix: String, schema: TSchema): String
  }

  object NameGenerator {
    private case object HashCode extends NameGenerator {
      def make(prefix: String, schema: TSchema): String = s"${prefix}_${hashCode()}"
    }

    private case object Prefix extends NameGenerator {
      def make(prefix: String, schema: TSchema): String = prefix
    }

    final private case class Incremental(int: AtomicInteger) extends NameGenerator {
      override def make(prefix: String, schema: TSchema): String = s"${prefix}_${int.incrementAndGet().toString}"
    }

    def incremental: NameGenerator = Incremental(new AtomicInteger(0))
    def schemaHash: NameGenerator  = HashCode
    def prefixOnly: NameGenerator  = Prefix
  }
}
