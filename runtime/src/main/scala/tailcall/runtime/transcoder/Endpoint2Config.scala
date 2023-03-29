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
    def toConfig(endpoint: Endpoint): TValid[String, Config] =
      for {
        baseURL <- toBaseURL(endpoint)
        graphQL <- toGraphQL(endpoint)
      } yield Config(server = Server(baseURL = Option(baseURL)), graphQL = graphQL)

    private def getInputTypeName(schema: TSchema): String = nameGen.gen("InputType", schema)

    private def getTypeName(schema: TSchema): String = nameGen.gen("Type", schema)

    private def toArgument(schema: TSchema, isRequired: Boolean, isList: Boolean): Config.Argument =
      schema match {
        case schema @ TSchema.Obj(_)  => Config
            .Argument(typeOf = getInputTypeName(schema), isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Arr(schema)      => toArgument(schema, isRequired = isRequired, isList = true)
        case TSchema.Optional(schema) => toArgument(schema, isRequired = false, isList = isList)
        case TSchema.String           => Config
            .Argument(typeOf = "String", isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Int => Config.Argument(typeOf = "Int", isRequired = Option(isRequired), isList = Option(isList))
        case TSchema.Boolean => Config
            .Argument(typeOf = "Boolean", isRequired = Option(isRequired), isList = Option(isList))
      }

    private def toArgumentMap(schema: TSchema, isRequired: Boolean, isList: Boolean): Map[String, Config.Argument] = {
      schema match {
        case TSchema.Obj(fields) => fields.map { field =>
            val name = field.name
            val arg  = toArgument(field.schema, isRequired = true, isList = false)
            name -> arg
          }.toMap

        case TSchema.Arr(item)        => toArgumentMap(item, isRequired = false, isList = true)
        case TSchema.Optional(schema) => toArgumentMap(schema, isRequired = false, isList = isList)
        case TSchema.String           =>
          Map("value" -> Config.Argument(typeOf = "String", isRequired = Option(isRequired), isList = Option(isList)))
        case TSchema.Int              =>
          Map("value" -> Config.Argument(typeOf = "Int", isRequired = Option(isRequired), isList = Option(isList)))
        case TSchema.Boolean          =>
          Map("value" -> Config.Argument(typeOf = "Boolean", isRequired = Option(isRequired), isList = Option(isList)))
      }
    }

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

    private def toGraphQL(endpoint: Endpoint): TValid[String, Config.GraphQL] =
      TValid.succeed {
        val rootSchema = RootSchema(query = Option("Query"), mutation = Option("Mutation"))
        val rootTypes  =
          if (endpoint.method == Method.GET) Map("Query" -> toRootTypeField(endpoint).toList)
          else Map("Mutation"                            -> toRootTypeField(endpoint).toList, "Query" -> List.empty)
        val types      = endpoint.output.map(toTypes(_, isRequired = true, isList = false)).getOrElse(Nil) ++ rootTypes
        GraphQL(schema = rootSchema, types = types.map { case (key, value) => key -> value.toMap }.toMap)
      }

    private def toRootTypeField(endpoint: Endpoint): Option[(String, Config.Field)] = {
      endpoint.output.map(schema => {
        val config = toConfigField(schema, isRequired = true, isList = false)
          .withSteps(List(Http.fromEndpoint(endpoint).withOutput(None))).compress

        val config0 = endpoint.input match {
          case Some(schema) => config.withArguments(toArgumentMap(schema, isRequired = true, isList = false))
          case None         => config
        }
        s"field${config0.typeOf}" -> config0
      })
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

  trait NameGenerator  {
    final private var cache = Map.empty[TSchema, String]

    final def gen(prefix: String, schema: TSchema): String = {
      cache.get(schema) match {
        case Some(name) => name
        case None       =>
          val name = unsafeGen(prefix, schema)
          cache = cache.updated(schema, name)
          name
      }
    }

    def unsafeGen(prefix: String, schema: TSchema): String
  }
  object NameGenerator {
    def incremental: NameGenerator = Incremental(new AtomicInteger(0))
    def prefixOnly: NameGenerator  = Prefix
    def schemaHash: NameGenerator  = HashCode

    final private case class Incremental(int: AtomicInteger) extends NameGenerator {
      override def unsafeGen(prefix: String, schema: TSchema): String = s"${prefix}_${int.incrementAndGet().toString}"
    }

    private case object HashCode extends NameGenerator {
      def unsafeGen(prefix: String, schema: TSchema): String = s"${prefix}_${hashCode()}"
    }

    private case object Prefix extends NameGenerator {
      def unsafeGen(prefix: String, schema: TSchema): String = prefix
    }
  }
}
