package tailcall.gateway.service

import caliban.introspection.adt._
import tailcall.gateway.ast.Graph
import zio.schema.meta.ExtensibleMetaSchema
import zio.{ZIO, ZLayer}

import scala.collection.mutable

trait TypeGenerator {
  def __type(graph: Graph): __Type
}

object TypeGenerator {
  def __type(graph: Graph): ZLayer[TypeGenerator, Nothing, __Type] =
    ZLayer(ZIO.serviceWith[TypeGenerator](_.__type(graph)))

  def live: ZLayer[Any, Nothing, TypeGenerator] = ZLayer.succeed(new Live())

  final class Live extends TypeGenerator {
    self =>

    private val pending: mutable.Map[zio.schema.TypeId, Graph] = mutable.Map.empty

    override def __type(graph: Graph): __Type = {
      graph.fields.map(cons => cons.fromType.ast -> cons).collect {
        case (product: ExtensibleMetaSchema.Product[_], cons) => product.id -> cons
      }.foreach { case (id, cons) =>
        pending.get(id) match {
          case None        => pending += (id -> cons.toGraph)
          case Some(value) => pending += id  -> (cons :: value)
        }
      }

      __Type(
        kind = __TypeKind.OBJECT,
        name = Some("Query"),
        fields = _ =>
          Some(
            graph.fields.filter(_.fromType == zio.schema.Schema[Unit])
              .map(cons => __Field(cons.name, None, Nil, () => generateType(cons.toType.ast)))
          )
      )
    }

    private def generateType(meta: zio.schema.meta.MetaSchema): __Type = {

      meta match {
        case ExtensibleMetaSchema.Product(id, _, fields, optional) =>
          val oldFields: List[__Field] = pending.get(id) match {
            case None        => Nil
            case Some(value) =>
              pending -= id
              value.fields.map(cons => __Field(cons.name, None, Nil, () => generateType(cons.toType.ast)))
          }
          notNull(
            __Type(
              kind = __TypeKind.OBJECT,
              name = Some(id.name),
              fields = _ =>
                Some(
                  fields.map(field => __Field(field.label, None, Nil, () => generateType(field.schema)))
                    .toList ++ oldFields
                )
            ),
            !optional
          )

        case ExtensibleMetaSchema.ListNode(item, _, optional) =>
          notNull(__Type(kind = __TypeKind.LIST, name = None, ofType = Some(generateType(item))), !optional)

        case ExtensibleMetaSchema.Value(valueType, _, optional) =>
          notNull(__Type(__TypeKind.SCALAR, name = Some(valueType.tag.capitalize)), !optional)

        // TODO: implement the rest of the cases
        // case ExtensibleMetaSchema.Ref(refPath, path, optional)             => ???
        // case ExtensibleMetaSchema.Dynamic(withSchema, path, optional)      => ???
        // case ExtensibleMetaSchema.Tuple(path, left, right, optional)       => ???
        // case ExtensibleMetaSchema.FailNode(message, path, optional)        => ???
        // case ExtensibleMetaSchema.Either(path, left, right, optional)      => ???
        // case ExtensibleMetaSchema.Dictionary(keys, values, path, optional) => ???
        // case ExtensibleMetaSchema.Sum(id, path, cases, optional)           => ???

        case schema => throw new MatchError(schema)
      }
    }

    private def notNull(t: __Type, cond: Boolean): __Type = if (cond) t.nonNull else t

  }
}
