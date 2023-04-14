package tailcall.runtime.transcoder

import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Config.{GraphQL, RootSchema}
import tailcall.runtime.model.Step.Http
import tailcall.runtime.model.{Config, Endpoint, Server, TSchema}
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

    private def getTypeName(schema: TSchema): String = nameGen.gen("Type", schema)

    private def toArgument(schema: TSchema, isRequired: Boolean, isList: Boolean): Config.Arg = {
      schema match {
        case schema @ TSchema.Obj(_) => Config
            .Arg(typeOf = getTypeName(schema), required = Option(isRequired), list = Option(isList))
        case TSchema.Arr(schema)     => toArgument(schema, isRequired = isRequired, isList = true)
        case TSchema.Opt(schema)     => toArgument(schema, isRequired = false, isList = isList)
        case TSchema.Str  => Config.Arg(typeOf = "String", required = Option(isRequired), list = Option(isList))
        case TSchema.Num  => Config.Arg(typeOf = "Int", required = Option(isRequired), list = Option(isList))
        case TSchema.Bool => Config.Arg(typeOf = "Boolean", required = Option(isRequired), list = Option(isList))
        case schema @ TSchema.Dictionary(_) =>
          toArgument(schema.toKeyValueArr, isRequired = isRequired, isList = isList)
      }
    }

    private def toArgumentMap(schema: TSchema, isRequired: Boolean, isList: Boolean): Map[String, Config.Arg] = {
      schema match {
        case TSchema.Obj(_)      => Map("value" -> toArgument(schema, isRequired = isRequired, isList = isList))
        case TSchema.Arr(item)   => toArgumentMap(item, isRequired = false, isList = true)
        case TSchema.Opt(schema) => toArgumentMap(schema, isRequired = false, isList = isList)
        case TSchema.Str         => Map("value" -> toArgument(schema, isRequired = isRequired, isList = isList))
        case TSchema.Num         => Map("value" -> toArgument(schema, isRequired = isRequired, isList = isList))
        case TSchema.Bool        => Map("value" -> toArgument(schema, isRequired = isRequired, isList = isList))
        case schema @ TSchema.Dictionary(_) =>
          toArgumentMap(schema.toKeyValueArr, isRequired = isRequired, isList = isList)
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
        case TSchema.Obj(_)                 => Config
            .Field(typeOf = getTypeName(schema), required = Option(isRequired), list = Option(isList))
        case TSchema.Arr(schema)            => toConfigField(schema, isRequired, isList = true)
        case TSchema.Opt(schema)            => toConfigField(schema, isRequired = false, isList = isList)
        case schema @ TSchema.Dictionary(_) =>
          toConfigField(schema.toKeyValueArr, isRequired = isRequired, isList = isList)
        case TSchema.Str  => Config.Field(typeOf = "String", required = Option(isRequired), list = Option(isList))
        case TSchema.Num  => Config.Field(typeOf = "Int", required = Option(isRequired), list = Option(isList))
        case TSchema.Bool => Config.Field(typeOf = "Boolean", required = Option(isRequired), list = Option(isList))
      }
    }

    private def toFields(fields: Map[String, TSchema]): Map[String, Config.Field] = {
      fields.map(field => field._1 -> toConfigField(field._2, isRequired = true, isList = false))
    }

    private def toGraphQL(endpoint: Endpoint): TValid[String, Config.GraphQL] =
      TValid.succeed {
        val rootSchema = RootSchema(query = Option("Query"), mutation = Option("Mutation"))

        val rootTypes =
          if (endpoint.method == Method.GET) Map("Query" -> toRootTypeField(endpoint).toList)
          else Map("Mutation"                            -> toRootTypeField(endpoint).toList, "Query" -> List.empty)

        val outputTypes = endpoint.output.map(toTypes(_, isRequired = true, isList = false)).getOrElse(Nil)
        val inputTypes  = endpoint.input.map(toTypes(_, isRequired = true, isList = false)).getOrElse(Nil)
        val types       = inputTypes ++ outputTypes ++ rootTypes
        GraphQL(
          schema = rootSchema,
          types = types.map { case (key, value) =>
            key -> Config.Type(doc = endpoint.description, fields = value.toMap)
          }.toMap,
        )
      }

    private def toRootTypeField(endpoint: Endpoint): Option[(String, Config.Field)] = {
      endpoint.output.map(schema => {
        var config = toConfigField(schema, isRequired = true, isList = false)
          .withSteps(Http.fromEndpoint(endpoint).withInput(None).withOutput(None))

        config = endpoint.input match {
          case Some(schema) => config.withArguments(toArgumentMap(schema, isRequired = true, isList = false))
          case None         => config
        }
        s"field${config.typeOf}" -> config
      })
    }

    private def toTypes(
      schema: TSchema,
      isRequired: Boolean,
      isList: Boolean,
    ): Map[String, Map[String, Config.Field]] = {
      schema match {
        case TSchema.Obj(fields)            =>
          val head = getTypeName(schema) -> toFields(fields)
          val tail = fields.flatMap(field => toTypes(field._2, isRequired, isList))
          tail + head
        case TSchema.Arr(item)              => toTypes(item, isRequired, isList = true)
        case TSchema.Opt(schema)            => toTypes(schema, isRequired = false, isList = isList)
        case TSchema.Str                    => Map.empty
        case TSchema.Num                    => Map.empty
        case TSchema.Bool                   => Map.empty
        case schema @ TSchema.Dictionary(_) => toTypes(schema.toKeyValueArr, isRequired = isRequired, isList = isList)
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
