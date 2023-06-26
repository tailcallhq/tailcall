package tailcall.runtime.model

import tailcall.runtime.JsonT
import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda.{Lambda, ~>>}
import tailcall.runtime.model.Config._
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.service.ConfigFileIO
import tailcall.runtime.transcoder.Transcoder
import zio.ZIO
import zio.json._
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}

import java.io.File
import java.net.{URI, URL}

/**
 * A configuration class for generating a GraphQL server.
 *
 * @param version:
 *   The version of the config.
 * @param server:
 *   The server configuration.
 * @param graphQL:
 *   The GraphQL configuration.
 */
final case class Config(version: Int = 0, server: Server = Server(), graphQL: GraphQL = GraphQL()) {
  self =>

  /**
   * Creates a new Config object with types specified.
   *
   * @param input
   *   : A variable number of tuple pairs (String, Type).
   * @return
   *   A new Config instance with the specified types.
   */
  def apply(input: (String, Type)*): Config = withTypes(input: _*)

  /**
   * Compresses the GraphQL and server configurations.
   *
   * @return
   *   A new Config instance with compressed GraphQL and
   *   server configurations.
   */
  def compress: Config = self.copy(graphQL = self.graphQL.compress, server = self.server.compress)

  /**
   * Retrieves a specific type in the current GraphQL
   * configuration.
   *
   * @param name
   *   : The name of the type to retrieve.
   * @return
   *   An Option containing the Type if it exists.
   */
  def findType(name: String): Option[Config.Type] = { self.graphQL.types.get(name) }

  /**
   * Retrieves all input types in the current GraphQL
   * configuration.
   *
   * @return
   *   A list of input types as strings.
   */
  def inputTypes: List[String] = {
    def loop(name: String, returnTypes: List[String]): List[String] = {
      if (returnTypes.contains(name)) returnTypes
      else findType(name) match {
        case Some(typeInfo) => for {
            fields <- typeInfo.fields.values.toList
            types  <- loop(fields.typeOf, name :: returnTypes)
          } yield types
        case None           => returnTypes
      }
    }

    for {
      typeInfo <- self.graphQL.types.values.toList
      field    <- typeInfo.fields.values.toList
      arg      <- field.args.getOrElse(Map.empty).values.toList
      types    <- loop(arg.typeOf, Nil)
    } yield types
  }

  /**
   * Merges the current Config instance with another, taking
   * the right-side values when conflicts occur.
   *
   * @param other
   *   : The other Config instance to merge with.
   * @return
   *   A new Config instance that is the result of the
   *   merge.
   */
  def mergeRight(other: Config): Config = {
    val newVersion = other.version match {
      case 0 => self.version
      case _ => other.version
    }

    val newServer = Server(
      baseURL = other.server.baseURL.orElse(self.server.baseURL),
      vars = other.server.vars.orElse(self.server.vars),
    )

    val newGraphQL = other.graphQL.mergeRight(self.graphQL)

    Config(version = newVersion, server = newServer, graphQL = newGraphQL)
  }

  /**
   * Returns the type information for mutation.
   * @return
   *   The type information for mutation.
   */
  def mutationType: Option[Type] = graphQL.schema.query.flatMap(findType(_))

  /**
   * Identifies potential N + 1 fanouts in the current
   * configuration
   * @return
   *   A list of paths to N + 1 fanouts.
   */
  def nPlusOne: List[List[(String, String)]] = {
    def findFanOut(
      typeName: String,
      path: Vector[(String, String)],
      isList: Boolean,
    ): List[Vector[(String, String)]] = {
      findType(typeName) match {
        case Some(typeOf) => typeOf.fields.toList.flatMap { case (name, field) =>
            val newPath = path :+ (typeName, name)
            if (path.contains((typeName, name))) List.empty
            else if (field.hasResolver && !field.hasBatchedResolver && isList) List(newPath)
            else findFanOut(field.typeOf, newPath, field.isList || isList)
          }
        case None         => List.empty
      }
    }

    graphQL.schema.query.toList.flatMap(findFanOut(_, Vector.empty, false).map(_.toList))
  }

  /**
   * Retrieves all output types in the current GraphQL
   * configuration.
   *
   * @return
   *   A list of output types as strings.
   */
  def outputTypes: List[String] = {
    def loop(name: String, result: List[String]): List[String] = {
      if (result.contains(name)) result
      else findType(name) match {
        case Some(typeInfo) => typeInfo.fields.values.toList
            .flatMap[String](field => loop(field.typeOf, name :: result))
        case None           => result
      }
    }

    val types = self.graphQL.schema.query.toList ++ self.graphQL.schema.mutation.toList
    types ++ types.foldLeft(List.empty[String]) { case (list, name) => loop(name, list) }
  }

  /**
   * Returns the type information for query.
   * @return
   *   The type information for query.
   */
  def queryType: Option[Type] = graphQL.schema.query.flatMap(findType(_))

  /**
   * Transforms the current Config instance into a
   * Blueprint.
   *
   * @return
   *   A TValid instance that either contains an error
   *   string or the Blueprint.
   */
  def toBlueprint: TValid[String, Blueprint] = Transcoder.toBlueprint(self)

  /**
   * Lists the type name and the field name where an unsafe
   * step is defined.
   *
   * @return
   *   Name of the type and the field in a tuple.
   */
  def unsafeSteps: List[(String, String)] =
    self.graphQL.types.toList.flatMap { case (typeName, typeOf) =>
      typeOf.fields.toList
        .collect { case (fieldName, field) if field.unsafeSteps.exists(_.nonEmpty) => (typeName, fieldName) }
    }

  /**
   * Creates a new Config instance with a specified baseURL.
   *
   * @param url
   *   : The URL as a URL instance.
   * @return
   *   A new Config instance with the specified baseURL.
   */
  def withBaseURL(url: URL): Config = self.copy(server = self.server.copy(baseURL = Option(url)))

  /**
   * Creates a new Config instance with a specified baseURL.
   *
   * @param url
   *   : The URL as a string.
   * @return
   *   A new Config instance with the specified baseURL.
   */
  def withBaseURL(url: String): Config = self.copy(server = self.server.copy(baseURL = Option(URI.create(url).toURL)))

  /**
   * Creates a new Config instance with a specified
   * mutation.
   *
   * @param mutation
   *   : The mutation as a string.
   * @return
   *   A new Config instance with the specified mutation.
   */
  def withMutation(mutation: String): Config = self.copy(graphQL = self.graphQL.withMutation(mutation))

  /**
   * Creates a new Config instance with a specified query.
   *
   * @param query
   *   : The query as a string.
   * @return
   *   A new Config instance with the specified query.
   */
  def withQuery(query: String): Config = self.copy(graphQL = self.graphQL.withQuery(query))

  /**
   * Creates a new Config instance with a specified root
   * schema.
   *
   * @param query
   *   : The query as an Option. Default is the current
   *   GraphQL schema's query.
   * @param mutation
   *   : The mutation as an Option. Default is the current
   *   GraphQL schema's mutation.
   * @return
   *   A new Config instance with the specified root schema.
   */
  def withRootSchema(
    query: Option[String] = graphQL.schema.query,
    mutation: Option[String] = graphQL.schema.mutation,
  ): Config = self.copy(graphQL = self.graphQL.copy(schema = RootSchema(query, mutation)))

  /**
   * Creates a new Config instance with specified types.
   *
   * @param types
   *   : A map of types.
   * @return
   *   A new Config instance with the specified types.
   */
  def withTypes(types: Map[String, Type]): Config =
    copy(graphQL = self.graphQL.copy(types = mergeTypeMap(self.graphQL.types, types)))

  /**
   * Creates a new Config instance with specified types.
   *
   * @param input
   *   : A variable number of tuple pairs (String, Type).
   * @return
   *   A new Config instance with the specified types.
   */
  def withTypes(input: (String, Type)*): Config = withTypes(input.toMap)

  /**
   * Creates a new Config instance with specified variables.
   *
   * @param vars
   *   : A variable number of tuple pairs (String, String).
   * @return
   *   A new Config instance with the specified variables.
   */
  def withVars(vars: (String, String)*): Config = self.copy(server = self.server.copy(vars = Option(vars.toMap)))
}

object Config {
  implicit lazy val typeInfoCodec: JsonCodec[Type]               = DeriveJsonCodec.gen[Type]
  implicit lazy val inputTypeCodec: JsonCodec[Arg]               = DeriveJsonCodec.gen[Arg]
  implicit lazy val fieldAnnotationCodec: JsonCodec[ModifyField] = DeriveJsonCodec.gen[ModifyField]
  implicit lazy val fieldDefinitionCodec: JsonCodec[Field]       = DeriveJsonCodec.gen[Field]
  implicit lazy val schemaDefinitionCodec: JsonCodec[RootSchema] = DeriveJsonCodec.gen[RootSchema]
  implicit lazy val graphQLCodec: JsonCodec[GraphQL]             = DeriveJsonCodec.gen[GraphQL]
  implicit lazy val jsonCodec: JsonCodec[Config]                 = DeriveJsonCodec.gen[Config]

  def default: Config = Config.empty.withQuery("Query")

  def empty: Config = Config()

  def fromFile(file: File): ZIO[ConfigFileIO, Throwable, Config] = ConfigFileIO.readFile(file)

  private def compressOptional[A](data: Option[List[A]]): Option[List[A]] =
    data match {
      case Some(Nil) => None
      case data      => data
    }

  private def mergeTypeMap(m1: Map[String, Type], m2: Map[String, Type]) = {
    (for {
      key    <- m1.keys ++ m2.keys
      typeOf <- (m1.get(key), m2.get(key)) match {
        case (Some(t1), Some(t2)) => List(t1.mergeRight(t2))
        case (t1, t2)             => t2.orElse(t1).toList
      }
    } yield key -> typeOf).toMap
  }

  final case class RootSchema(query: Option[String] = None, mutation: Option[String] = None) {
    def mergeRight(other: RootSchema): RootSchema =
      RootSchema(query = other.query.orElse(query), mutation = other.mutation.orElse(mutation))
  }

  final case class Type(
    doc: Option[String] = None,
    fields: Map[String, Field] = Map.empty,
    // FIXME: keep it as Option[String]
    // FIXME: validate if the type extends itself
    @jsonField("extends") baseType: Option[List[String]] = None,
  ) {
    self =>
    def apply(input: (String, Field)*): Type = withFields(input: _*)

    def compress: Type =
      self.copy(
        fields = self.fields.toSeq.sortBy(_._1).map { case (k, v) => k -> v.compress }.toMap,
        baseType = compressOptional(self.baseType),
      )

    def extendsWith(types: String*): Type = self.copy(baseType = Option(types.toList))

    def mergeRight(other: Config.Type): Config.Type = {
      val newFields = other.fields ++ self.fields
      Config.Type(doc = other.doc.orElse(self.doc), fields = newFields)
    }

    def withDoc(doc: String): Type = self.copy(doc = Option(doc))

    def withField(name: String, field: Field): Type = self.copy(fields = self.fields + (name -> field))

    def withFields(input: (String, Field)*): Type =
      input.foldLeft(self) { case (self, (name, field)) => self.withField(name, field) }
  }

  final case class GraphQL(schema: RootSchema = RootSchema(), types: Map[String, Type] = Map.empty) {
    self =>
    def compress: GraphQL =
      self.copy(types = self.types.toSeq.sortBy(_._1).map { case (k, t) => (k, t.compress) }.toMap)

    def mergeRight(other: GraphQL): GraphQL =
      GraphQL(schema = self.schema.mergeRight(other.schema), types = mergeTypeMap(self.types, other.types))

    def withMutation(name: String): GraphQL = copy(schema = schema.copy(mutation = Option(name)))

    def withQuery(name: String): GraphQL = copy(schema = schema.copy(query = Option(name)))

    def withSchema(query: Option[String], mutation: Option[String]): GraphQL =
      copy(schema = RootSchema(query, mutation))
  }

  final case class Field(
    @jsonField("type") typeOf: String,
    @jsonField("isList") list: Option[Boolean] = None,
    @jsonField("isRequired") required: Option[Boolean] = None,
    unsafeSteps: Option[List[Operation]] = None,
    args: Option[Map[String, Arg]] = None,
    doc: Option[String] = None,
    modify: Option[ModifyField] = None,
    http: Option[Http] = None,
    inline: Option[InlineType] = None,
  ) {
    self =>

    def apply(args: (String, Arg)*): Field = copy(args = Option(args.toMap))

    def asList: Field = copy(list = Option(true))

    def asRequired: Field = copy(required = Option(true))

    def compress: Field = {
      val steps = self.unsafeSteps match {
        case Some(steps) if steps.nonEmpty => Option(steps.map(_.compress))
        case _                             => None
      }

      val args = self.args match {
        case Some(args) if args.nonEmpty => Some(args.map { case (k, v) => (k, v.compress) })
        case _                           => None
      }

      copy(
        list = self.list.filter(_ == true),
        required = self.required.filter(_ == true),
        unsafeSteps = steps,
        args = args,
        modify = self.modify.filter(_.nonEmpty),
        http = http.map(_.compress),
        inline = self.inline.filter(_.path.nonEmpty),
      )
    }

    def hasBatchedResolver: Boolean =
      http match {
        case Some(http) => http.batchKey.nonEmpty && http.groupBy.nonEmpty
        case None       => false
      }

    def hasResolver: Boolean = http.isDefined || unsafeSteps.exists(_.nonEmpty)

    def isList: Boolean = list.getOrElse(false)

    def isRequired: Boolean = required.getOrElse(false)

    def resolveWith[A: Schema](a: A): Field = resolveWithFunction(_ => Lambda(DynamicValue(a)))

    def resolveWithFunction(f: DynamicValue ~>> DynamicValue): Field = withSteps(Operation.function(f))

    def resolveWithJson[A: JsonEncoder](a: A): Field = withSteps(Operation.constant(a.toJsonAST.toOption.get))

    def withArguments(args: (String, Arg)*): Field = withArguments(args.toMap)

    def withArguments(args: Map[String, Arg]): Field = copy(args = Option(args))

    def withDoc(doc: String): Field = copy(doc = Option(doc))

    def withHttp(http: Http): Field = copy(http = Option(http))

    def withHttp(
      path: Path,
      method: Option[Method] = None,
      query: Map[String, String] = Map.empty,
      input: Option[TSchema] = None,
      output: Option[TSchema] = None,
      body: Option[String] = None,
      groupBy: List[String] = Nil,
      batchKey: Option[String] = None,
    ): Field = withHttp(Http(path, method, Option(query), input, output, body, Option(groupBy), batchKey))

    def withInline(path: String*): Field = copy(inline = Option(InlineType(path.toList)))

    def withJsonT(head: JsonT, tail: JsonT*): Field =
      withSteps {
        val all = head :: tail.toList
        Operation.transform(all.reduce(_ >>> _))
      }

    def withName(name: String): Field = withUpdate(ModifyField.empty.withName(name))

    def withOmit(omit: Boolean): Field = withUpdate(ModifyField.empty.withOmit(omit))

    def withSteps(steps: Operation*): Field = copy(unsafeSteps = Option(steps.toList))

    def withUpdate(update: ModifyField): Field = {
      copy(modify = self.modify match {
        case Some(value) => Some(value mergeRight update)
        case None        => Some(update)
      })
    }

  }

  final case class Arg(
    @jsonField("type") typeOf: String,
    @jsonField("isList") list: Option[Boolean] = None,
    @jsonField("isRequired") required: Option[Boolean] = None,
    doc: Option[String] = None,
    modify: Option[ModifyField] = None,
    @jsonField("default") defaultValue: Option[Json] = None,
  ) {
    self =>
    def asList: Arg = self.copy(list = Option(true))

    def asRequired: Arg = self.copy(required = Option(true))

    def compress: Arg = {
      val isList = self.list match {
        case Some(true) => Some(true)
        case _          => None
      }

      val isRequired = self.required match {
        case Some(true) => Some(true)
        case _          => None
      }

      val update = self.modify match {
        case Some(value) if value.nonEmpty => Some(value)
        case _                             => None
      }

      self.copy(list = isList, required = isRequired, modify = update)
    }

    def isList: Boolean = list.getOrElse(false)

    def isRequired: Boolean = required.getOrElse(false)

    def withDefault[A: JsonEncoder](value: A): Arg = copy(defaultValue = value.toJsonAST.toOption)

    def withDoc(doc: String): Arg = copy(doc = Option(doc))

    def withName(name: String): Arg = withUpdate(ModifyField.empty.withName(name))

    def withUpdate(update: ModifyField): Arg =
      copy(modify = self.modify match {
        case Some(value) => Some(value mergeRight update)
        case None        => Some(update)
      })
  }

  object Type {
    def apply(fields: (String, Field)*): Type = Type(None, fields.toMap)
    def empty: Type                           = Type(None, Map.empty[String, Field])
  }

  object Field {
    def apply(str: String, operations: Operation*): Field =
      Field(typeOf = str, unsafeSteps = if (operations.isEmpty) None else Option(operations.toList))

    def bool: Field = Field(typeOf = "Boolean")

    def int: Field = Field(typeOf = "Int")

    def ofType(name: String): Field = Field(typeOf = name)

    def str: Field = Field(typeOf = "String")
  }

  object Arg {
    val string: Arg               = Arg("String")
    val int: Arg                  = Arg("Int")
    val bool: Arg                 = Arg("Boolean")
    def ofType(name: String): Arg = Arg(name)
  }
}
