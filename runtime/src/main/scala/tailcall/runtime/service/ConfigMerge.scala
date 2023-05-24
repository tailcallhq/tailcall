package tailcall.runtime.service

import tailcall.runtime.model.{Config, Server}

object ConfigMerge {

  def mergeAll(configs: List[Config]): Config = configs.reduce(mergeRight)

  /**
   * Merge two configs, with the right config taking
   * precedence.
   */
  def mergeRight(c1: Config, c2: Config): Config = {
    val newVersion = c2.version match {
      case 0 => c1.version
      case _ => c2.version
    }

    val newServer =
      Server(baseURL = c2.server.baseURL.orElse(c1.server.baseURL), vars = c2.server.vars.orElse(c1.server.vars))

    val newGraphQL = Config.GraphQL(
      schema = Config.RootSchema(
        query = c2.graphQL.schema.query.orElse(c1.graphQL.schema.query),
        mutation = c2.graphQL.schema.mutation.orElse(c1.graphQL.schema.mutation),
      ),
      types = mergeTypes(c2.graphQL.types ++ c1.graphQL.types),
    )

    Config(version = newVersion, server = newServer, graphQL = newGraphQL)
  }

  def mergeTypes(types: List[Config.Type]): List[Config.Type] = {
    types.groupBy(t => (t.isInput, t.name)).values.map(_.reduce(mergeTypes)).toList
  }

  /**
   * Merge two types, with the right type taking precedence
   * assuming they have the same name and input type.
   */
  private def mergeTypes(t1: Config.Type, t2: Config.Type): Config.Type = {
    val newFields = t2.fields ++ t1.fields
    Config.Type(name = t2.name, doc = t2.doc.orElse(t1.doc), input = t2.input.orElse(t1.input), fields = newFields)
  }
}
