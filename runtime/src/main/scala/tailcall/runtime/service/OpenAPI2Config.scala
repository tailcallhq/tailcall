package tailcall.runtime.service

import tailcall.runtime.ast.{Path, TSchema}
import tailcall.runtime.dsl.json.Config.Step
import tailcall.runtime.openApi.OpenapiModels.OpenapiDocument
import tailcall.runtime.http.Method
import tailcall.runtime.openApi.OpenapiSchemaType
import io.circe.syntax._
import io.circe.generic.auto._

object OpenAPI2Config {

  def getPath(doc: OpenapiDocument, path: List[String]): Option[TSchema]          = {
    pprint.pprintln(path)
    path match {
      case ::(head, tail) =>
        pprint.pprintln(doc.asJson)
        doc.asJson.hcursor.downField(head).as[Map[String, OpenapiSchemaType]] match {
          case Right(value) => value.get(tail.head) match {
              case Some(value) => Option(openapiSchema2TSchema(doc, value))
              case None        => None
            }
          case Left(_)      => None
        }
      case Nil            => None
    }
  }
  def openapiSchema2TSchema(doc: OpenapiDocument, in: OpenapiSchemaType): TSchema = {
    in match {
      case mixedType: OpenapiSchemaType.OpenapiSchemaMixedType     => mixedType match {
          case OpenapiSchemaType.OpenapiSchemaOneOf(types) => TSchema
              .union(types.map(openapiSchema2TSchema(doc, _)).toList)
          case OpenapiSchemaType.OpenapiSchemaAnyOf(types) => TSchema
              .union(types.map(openapiSchema2TSchema(doc, _)).toList)
          case OpenapiSchemaType.OpenapiSchemaAllOf(types) => TSchema
              .intersection(types.map(openapiSchema2TSchema(doc, _)).toList)
        }
      case simpleType: OpenapiSchemaType.OpenapiSchemaSimpleType   => simpleType match {
          case numericType: OpenapiSchemaType.OpenapiSchemaNumericType => numericType match {
              case OpenapiSchemaType.OpenapiSchemaDouble(_) => TSchema.int
              case OpenapiSchemaType.OpenapiSchemaFloat(_)  => TSchema.int
              case OpenapiSchemaType.OpenapiSchemaLong(_)   => TSchema.int
              case OpenapiSchemaType.OpenapiSchemaInt(_)    => TSchema.int
            }
          case _: OpenapiSchemaType.OpenapiSchemaStringType            => TSchema.string
          case OpenapiSchemaType.OpenapiSchemaBoolean(_)               => TSchema.bool
          case OpenapiSchemaType.OpenapiSchemaRef(name)                =>
            val key = name.split("/").takeRight(1).head
            doc.components.schemas.get(key) match {
              case Some(value) => TSchema.obj((key, openapiSchema2TSchema(doc, value)))
              case None        => TSchema.string
            }
        }
      case OpenapiSchemaType.OpenapiSchemaNot(_)                   => ???
      case OpenapiSchemaType.OpenapiSchemaArray(items, _)          => TSchema.arr(openapiSchema2TSchema(doc, items))
      case OpenapiSchemaType.OpenapiSchemaObject(properties, _, _) => TSchema.obj(properties.map {
          case (name, schema) => TSchema.Field(name, openapiSchema2TSchema(doc, schema))
        }.toList)
    }
  }
  def convert(openAPI: OpenapiDocument): List[Step.Http]                          = {
    val paths     = openAPI.paths
    val httpSteps = paths.flatMap(path => {
      val steps = path.methods.map(method => {
        val input: Option[TSchema]  = Option(TSchema.obj(
          method.parameters.map(param => TSchema.Field(param.name, openapiSchema2TSchema(openAPI, param.schema))).toList
        ))
        val output: Option[TSchema] = method.responses.find(_.code == "200").flatMap(response =>
          response.content.find(_.contentType == "application/json").map(x => openapiSchema2TSchema(openAPI, x.schema))
        )
        val step                    = Step.Http(
          path = Path.unsafe.fromString(path.url),
          method = Method.decode(method.methodType.toUpperCase).toOption,
          input,
          output
        )
        step
      })
      steps
    })
    pprint.pprintln(httpSteps)
    httpSteps.toList
  }

}
