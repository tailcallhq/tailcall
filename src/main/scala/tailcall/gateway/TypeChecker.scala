package tailcall.gateway

import caliban.parsing.adt.{Definition, Document}
import tailcall.gateway.dsl.json.Config

final class TypeChecker(config: Config, document: Document) {
  import tailcall.gateway.internal.CalibanADTOperators._
  def hasSchemaDefinition: TValid[String, Definition.TypeSystemDefinition.SchemaDefinition] = {
    document.definitions.collectFirst { case d: Definition.TypeSystemDefinition.SchemaDefinition => d } match {
      case Some(d) => TValid.success(d)
      case None    => TValid.fail("Missing schema definition")
    }
  }

  def hasResolverType(name: String): TValid[String, Map[String, Config.Connection]] = {
    config.graphQL.connections.get(name) match {
      case None        => TValid.fail(s"Missing resolver for type: $name")
      case Some(value) => TValid.success(value)
    }
  }

  def hasQueryType(
    schema: Definition.TypeSystemDefinition.SchemaDefinition
  ): TValid[String, Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition] = {
    schema.query.flatMap(document.findDefinition(_)) match {
      case None          => TValid.fail("Missing query in schema definition")
      case Some(objType) => TValid.success(objType)
    }
  }

  def checkResolverDefinition(
    objectType: Definition.TypeSystemDefinition.TypeDefinition.ObjectTypeDefinition
  ): TValid[String, Unit] = {
    for {
      resolverMap <- hasResolverType(objectType.name)
      schemaFields   = objectType.fields.map(_.name).toSet
      resolverFields = resolverMap.keySet
      diff           = schemaFields -- resolverFields
      _ <-
        if (schemaFields == resolverFields) TValid.empty
        else TValid.fail(s"Resolvers missing in type ${objectType.name}: ${diff.mkString(", ")}")
    } yield ()
  }

  def check: TValid[String, Unit] = {
    for {
      schema              <- hasSchemaDefinition
      queryTypeDefinition <- hasQueryType(schema)
      _                   <- checkResolverDefinition(queryTypeDefinition)
    } yield ()
  }
}

object TypeChecker {
  def check(config: Config, document: Document): TValid[String, Unit] = { new TypeChecker(config, document).check }
}
