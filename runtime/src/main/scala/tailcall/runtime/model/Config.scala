package tailcall.runtime.model

import tailcall.runtime.JsonT
import tailcall.runtime.http.Method
import tailcall.runtime.internal.TValid
import tailcall.runtime.lambda.{Lambda, ~>>}
import tailcall.runtime.model.Config._
import tailcall.runtime.model.UnsafeSteps.Operation
import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.service.{ConfigFileIO, ConfigMerge}
import tailcall.runtime.transcoder.Transcoder
import zio.json._
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema}
import zio.{IO, ZIO}

import java.io.File
import java.net.{URI, URL}

final case class Config(version: Int = 0, server: Server = Server(), graphQL: GraphQL = GraphQL()) {
  self =>

  def apply(input: (String, Type)*): Config = withTypes(input: _*)

  def asGraphQLConfig: IO[String, String] = ConfigFormat.GRAPHQL.encode(self)

  def asJSONConfig: IO[String, String] = ConfigFormat.JSON.encode(self)

  def asYAMLConfig: IO[String, String] = ConfigFormat.YML.encode(self)

  def compress: Config = self.copy(graphQL = self.graphQL.compress, server = self.server.compress)

  def getInputTypes: List[String] = {
    def loop(name: String, returnTypes: List[String]): List[String] = {
      if (returnTypes.contains(name)) returnTypes
      else getType(name) match {
        case Some(typeInfo) => for {
            fields <- typeInfo.fields.values.toList
            types  <- loop(fields.typeOf, name :: returnTypes)
          } yield types
        case None           => returnTypes
      }
    }

    for {
      typeInfo <- self.graphQL.types
      field    <- typeInfo.fields.values.toList
      arg      <- field.args.getOrElse(Map.empty).values.toList
      types    <- loop(arg.typeOf, Nil)
    } yield types
  }

  /**
   * Goes over every possible object type and creates a map
   * of type name to whether it's an input type or not.
   */
  def getOutputTypes: List[String] = {
    def loop(name: String, result: List[String]): List[String] = {
      if (result.contains(name)) result
      else getType(name) match {
        case Some(typeInfo) => typeInfo.fields.values.toList
            .flatMap[String](field => loop(field.typeOf, name :: result))
        case None           => result
      }
    }

    val types = self.graphQL.schema.query.toList ++ self.graphQL.schema.mutation.toList
    types ++ types.foldLeft(List.empty[String]) { case (list, name) => loop(name, list) }
  }

  def getType(name: String): Option[Config.Type] = { self.graphQL.types.find(_.name == name) }

  def mergeRight(other: Config): Config = ConfigMerge.mergeRight(self, other)

  def mergeTypes: List[Config.Type] = ConfigMerge.mergeTypes(self.graphQL.types)

  def toBlueprint: TValid[String, Blueprint] = Transcoder.toBlueprint(self)

  def unsafeCount: Int = self.graphQL.types.flatMap(_.fields.values.toList).count(_.unsafeSteps.nonEmpty)

  def withBaseURL(url: URL): Config = self.copy(server = self.server.copy(baseURL = Option(url)))

  def withBaseURL(url: String): Config = self.copy(server = self.server.copy(baseURL = Option(URI.create(url).toURL)))

  def withMutation(mutation: String): Config = self.copy(graphQL = self.graphQL.withMutation(mutation))

  def withQuery(query: String): Config = self.copy(graphQL = self.graphQL.withQuery(query))

  def withRootSchema(
    query: Option[String] = graphQL.schema.query,
    mutation: Option[String] = graphQL.schema.mutation,
  ): Config = self.copy(graphQL = self.graphQL.copy(schema = RootSchema(query, mutation)))

  def withTypes(input: (String, Type)*): Config = {
    self.copy(graphQL = self.graphQL.withTypes(input.map { case (name, typeInfo) => typeInfo.withName(name) }))
  }

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

  final case class RootSchema(query: Option[String] = None, mutation: Option[String] = None)

  final case class Type(name: String, doc: Option[String] = None, fields: Map[String, Field] = Map.empty) {
    self =>

    def apply(input: (String, Field)*): Type = withFields(input: _*)

    def compress: Type = self.copy(fields = self.fields.toSeq.sortBy(_._1).map { case (k, v) => k -> v.compress }.toMap)

    def withDoc(doc: String): Type = self.copy(doc = Option(doc))

    def withField(name: String, field: Field): Type = self.copy(fields = self.fields + (name -> field))

    def withFields(input: (String, Field)*): Type =
      input.foldLeft(self) { case (self, (name, field)) => self.withField(name, field) }

    def withName(name: String): Type = self.copy(name = name)
  }

  final case class GraphQL(schema: RootSchema = RootSchema(), types: List[Type] = List.empty) {
    self =>
    def compress: GraphQL = self.copy(types = self.types.sortBy(_.name).map(_.compress))

    def withMutation(name: String): GraphQL = copy(schema = schema.copy(mutation = Option(name)))

    def withQuery(name: String): GraphQL = copy(schema = schema.copy(query = Option(name)))

    def withSchema(query: Option[String], mutation: Option[String]): GraphQL =
      copy(schema = RootSchema(query, mutation))

    def withType(typeOf: Type): GraphQL = self.copy(types = typeOf :: self.types)

    def withTypes(value: Seq[Type]): GraphQL = copy(types = ConfigMerge.mergeTypes(types ++ value.toList))
  }

  // TODO: Field and Argument can be merged
  final case class Field(
    @jsonField("type") typeOf: String,
    list: Option[Boolean] = None,
    required: Option[Boolean] = None,
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
      val isList = self.list match {
        case Some(true) => Some(true)
        case _          => None
      }

      val isRequired = self.required match {
        case Some(true) => Some(true)
        case _          => None
      }

      val steps = self.unsafeSteps match {
        case Some(steps) if steps.nonEmpty => Option(steps.map(_.compress))
        case _                             => None
      }

      val args = self.args match {
        case Some(args) if args.nonEmpty => Some(args.map { case (k, v) => (k, v.compress) })
        case _                           => None
      }

      val modify = self.modify match {
        case Some(value) if value.nonEmpty => Some(value)
        case _                             => None
      }

      val inline = self.inline match {
        case Some(value) if value.path.nonEmpty => Some(value)
        case _                                  => None
      }

      copy(
        list = isList,
        required = isRequired,
        unsafeSteps = steps,
        args = args,
        modify = modify,
        http = http,
        inline = inline,
      )
    }

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
    ): Field = withHttp(Http(path, method, Option(query), input, output, body))

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
    def apply(fields: (String, Field)*): Type = empty.withFields(fields: _*)
    def empty: Type                           = Type(name = "$__PLACE_HOLDER__$")
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
